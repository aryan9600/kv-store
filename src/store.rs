use crate::{error::Result, KVStoreError};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::PathBuf,
    sync::Mutex,
};

// Ser/Derializable action to be stored in the log.
#[derive(Serialize, Deserialize, Debug)]
enum Action {
    Set { key: String, val: String },
    Remove { key: String },
}

// Pointer to a stored action in the log.
#[derive(Debug)]
struct ActionPointer {
    pos: u64,
    len: u64,
}

impl From<Range<u64>> for ActionPointer {
    fn from(range: Range<u64>) -> Self {
        ActionPointer {
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

// A BufReader along with a pointer to the file.
struct BufReaderWithPointer<R: Read + Seek> {
    reader: BufReader<R>,
    pointer: u64,
}

impl<R: Read + Seek> BufReaderWithPointer<R> {
    fn new(mut _inner: R) -> Result<Self> {
        let pointer = _inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPointer {
            reader: BufReader::new(_inner),
            pointer,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPointer<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pointer += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPointer<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pointer = self.reader.seek(pos)?;
        Ok(self.pointer)
    }
}

// A BufWriter along with a pointer to the file.
struct BufWriterWithPointer<W: Write + Seek> {
    writer: BufWriter<W>,
    pointer: u64,
}

impl<W: Write + Seek> BufWriterWithPointer<W> {
    fn new(mut _inner: W) -> Result<Self> {
        let pointer = _inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPointer {
            writer: BufWriter::new(_inner),
            pointer,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPointer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pointer += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPointer<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pointer = self.writer.seek(pos)?;
        Ok(self.pointer)
    }
}

/// KVStore provides methods to set, get and delete key-value pairs.
///
/// All mutable actions are stored in a log file for persistence.
/// A map stores the key along with a pointer, pointing to the file offset
/// where the latest action corresponding to the key is stored in the file.
///
/// ```rust
/// use kv_store::store::KVStore;
/// fn main() {
///     let log_path = "/tmp/store.log";
///     let store = KVStore::open(log_path).unwrap();
///     let key = String::from("this is");
///     let val = String::from("the way");
///     if let Some(value) = store.set(key.clone(), val).unwrap() {
///         println!("{}", value);
///     }
///     if let Some(value) = store.get(key).unwrap() {
///         println!("{}", value);
///     }
/// }
/// ```
pub struct KVStore {
    reader: Mutex<BufReaderWithPointer<File>>,
    writer: Mutex<BufWriterWithPointer<File>>,
    index: Mutex<BTreeMap<String, ActionPointer>>,
}

impl KVStore {
    /// Accepts a path to the open/create the log file. Parses the log file and load
    /// the keys and the pointers in the index. Returns a KVStore for use.
    pub fn open(path: impl Into<PathBuf>) -> Result<KVStore> {
        let path = path.into();
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?;

        let mut index = Mutex::new(BTreeMap::new());
        let mut reader = Mutex::new(BufReaderWithPointer::new(File::open(&path)?)?);
        let mut writer = Mutex::new(BufWriterWithPointer::new(log_file)?);

        let load_index = index.get_mut().map_err(|_| KVStoreError::Lock)?;
        let load_reader = reader.get_mut().map_err(|_| KVStoreError::Lock)?;
        let load_writer = writer.get_mut().map_err(|_| KVStoreError::Lock)?;
        load(load_reader, load_index, load_writer)?;

        Ok(KVStore {
            reader,
            index,
            writer,
        })
    }

    /// Stores the key and it's value. If the key already existed, the old value is returned.
    pub fn set(&self, key: String, val: String) -> Result<Option<String>> {
        let action_key = key.clone();
        let action = Action::Set {
            key: action_key,
            val,
        };
        let pointer = {
            let writer = &mut *self.writer.lock().map_err(|_| KVStoreError::Lock)?;
            let point = writer.pointer;
            serde_json::to_writer(writer, &action)?;
            point
        };
        {
            let writer = &mut *self.writer.lock().map_err(|_| KVStoreError::Lock)?;
            writer.flush()?;
        };
        let writer = &mut *self.writer.lock().map_err(|_| KVStoreError::Lock)?;
        let action_pointer: ActionPointer = (pointer..writer.pointer).into();
        let mut index = self.index.lock().map_err(|_| KVStoreError::Lock)?;
        if let Some(old_action_pointer) = index.insert(key, action_pointer) {
            let mut reader = self.reader.lock().map_err(|_| KVStoreError::Lock)?;
            reader.seek(SeekFrom::Start(old_action_pointer.pos))?;
            let mut buf = vec![0; old_action_pointer.len as usize];
            reader.read_exact(&mut buf)?;
            if let Action::Set { val, .. } = serde_json::from_slice(buf.as_slice())? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    /// Gets the value related to the given key. If not found, returns a KeyNotFound error.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let index = self.index.lock().map_err(|_| KVStoreError::Lock)?;
        if let Some(action_pointer) = index.get(&key) {
            let mut reader = self.reader.lock().map_err(|_| KVStoreError::Lock)?;
            reader.seek(SeekFrom::Start(action_pointer.pos))?;
            let mut buf = vec![0; action_pointer.len as usize];
            reader.read_exact(&mut buf)?;
            if let Action::Set { val, .. } = serde_json::from_slice(buf.as_slice())? {
                return Ok(Some(val));
            }
            return Ok(None);
        }
        Err(KVStoreError::KeyNotFound(key))
    }

    /// Removes a key and it's value from the store. Returns the current value of the key.
    /// If key is not found, returns a KeyNotFound error.
    pub fn rm(&self, key: String) -> Result<Option<String>> {
        let index = self.index.lock().map_err(|_| KVStoreError::Lock)?;
        if index.contains_key(&key) {
            let action = Action::Remove { key };
            {
                let writer = &mut *self.writer.lock().map_err(|_| KVStoreError::Lock)?;
                serde_json::to_writer(writer, &action)?;
            }
            {
                let writer = &mut *self.writer.lock().map_err(|_| KVStoreError::Lock)?;
                writer.flush()?;
            }
            if let Action::Remove { key } = action {
                std::mem::drop(index);
                println!("adsdad");
                let val = self.get(key.clone());
                let index = &mut *self.index.lock().map_err(|_| KVStoreError::Lock)?;
                index.remove(&key);
                return val;
            }
            Ok(None)
        } else {
            Err(KVStoreError::KeyNotFound(key))
        }
    }
}

// Parse the log file and populate the index.
fn load(
    reader: &mut BufReaderWithPointer<File>,
    index: &mut BTreeMap<String, ActionPointer>,
    writer: &mut BufWriterWithPointer<File>
) -> Result<()> {
    let mut pointer = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Action>();
    while let Some(action) = stream.next() {
        let new_pointer = stream.byte_offset() as u64;
        match action? {
            Action::Set { key, .. } => {
                let action_pointer: ActionPointer = (pointer..new_pointer).into();
                index.insert(key, action_pointer);
            }
            Action::Remove { key } => {
                index.remove(&key);
            }
        }
        pointer = new_pointer;
    }
    writer.seek(SeekFrom::Start(pointer))?;
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::Rng;
    use std::{fs, panic};

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(KVStore) -> () + panic::UnwindSafe,
    {
        // Since tests are run in parallel by default, we cannot have multiple unit tests working on the same file.
        let mut rng = rand::thread_rng();
        let n: u16 = rng.gen();
        let log_path = format!("/tmp/{}.log", n);
        let store = KVStore::open(log_path.clone()).unwrap();

        let result = panic::catch_unwind(move || test(store));

        fs::remove_file(log_path).unwrap();

        assert!(result.is_ok())
    }

    #[test]
    fn test_set() {
        run_test(|store: KVStore| {
            let key = String::from("this is");
            let val = String::from("the way");
            let res = store.set(key.clone(), val).unwrap();
            assert_eq!(res, None);
            let val = String::from("not the way");
            let res = store.set(key, val).unwrap();
            assert_eq!(res, Some(String::from("the way")));
        })
    }

    #[test]
    fn test_get() {
        run_test(|store: KVStore| {
            let key = String::from("this is");
            let val = String::from("the way");
            let res = store.set(key.clone(), val).unwrap();
            assert_eq!(res, None);
            let res = store.get(key).unwrap();
            assert_eq!(res, Some(String::from("the way")));
        })
    }

    #[test]
    fn test_rm() {
        run_test(|store: KVStore| {
            let key = String::from("this is");
            let val = String::from("the way");
            let res = store.set(key, val).unwrap();
            assert_eq!(res, None);
            let key = String::from("this is");
            let res = store.rm(key.clone()).unwrap();
            assert_eq!(res, Some(String::from("the way")));
        })
    }
}
