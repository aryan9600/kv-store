use std::{collections::BTreeMap, fs::{File, OpenOptions}, io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, ops::Range, path::PathBuf};
use crate::{KVStoreError, error::Result};
use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

#[derive(Serialize, Deserialize, Debug)]
enum Action {
    Set { key: String, val: String },
    Remove { key: String }
}

struct ActionPointer {
    pos: u64,
    len: u64
}

impl From<Range<u64>> for ActionPointer {
    fn from(range: Range<u64>) -> Self {
        ActionPointer { pos: range.start, len: range.end - range.start } 
    }
}

struct BufReaderWithPointer<R: Read + Seek> {
    reader: BufReader<R>,
    pointer: u64
}

impl<R: Read + Seek> BufReaderWithPointer<R> {
    fn new(mut _inner: R) -> Result<Self> {
        let pointer = _inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPointer{
            reader: BufReader::new(_inner),
            pointer
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

struct BufWriterWithPointer<W: Write + Seek> {
    writer: BufWriter<W>,
    pointer: u64
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

pub struct KVStore {
    reader: BufReaderWithPointer<File>,
    writer: BufWriterWithPointer<File>,
    index: BTreeMap<String, ActionPointer>,
}

impl KVStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KVStore> {
        let path = path.into();
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?;
        let writer = BufWriterWithPointer::new(log_file)?;

        let mut index = BTreeMap::new();
        let mut reader = BufReaderWithPointer::new(File::open(&path)?)?;
        load(&mut reader, &mut index)?;


        Ok(KVStore{
            reader,
            index,
            writer
        })
    }

    pub fn set(&mut self, key: String, val: String) -> Result<Option<String>>{
        let action_key = key.clone();
        let action = Action::Set{ key: action_key, val };
        let pointer = self.writer.pointer;
        serde_json::to_writer(&mut self.writer, &action)?;
        self.writer.flush()?;
        let action_pointer: ActionPointer = (pointer..self.writer.pointer).into();
        if let Some(old_action_pointer) = self.index.insert(key, action_pointer) {
            self.reader.seek(SeekFrom::Start(old_action_pointer.pos))?;
            let mut buf = vec![0; old_action_pointer.len as usize];
            self.reader.read_exact(&mut buf)?;
            if let Action::Set{ val, .. } = serde_json::from_slice(buf.as_slice())? {
                return Ok(Some(val))
            } 
        }
        Ok(None)
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(action_pointer) = self.index.get(&key) {
            self.reader.seek(SeekFrom::Start(action_pointer.pos))?;
            let mut buf = vec![0; action_pointer.len as usize];
            self.reader.read_exact(&mut buf)?;
            if let Action::Set{ val, .. } = serde_json::from_slice(buf.as_slice())? {
                return Ok(Some(val))
            } 
            return Ok(None)
        }
        
        Ok(None)
    }

    pub fn rm(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let action = Action::Remove{ key };
            serde_json::to_writer(&mut self.writer, &action)?;
            self.writer.flush()?;
            if let Action::Remove { key } = action {
                self.index.remove(&key);
            }
            Ok(())
        } else {
            Err(KVStoreError::KeyNotFound(key))
        }
    }
}

fn load(reader: &mut BufReaderWithPointer<File>, index: &mut BTreeMap<String, ActionPointer>) -> Result<()> {
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
    Ok(())
}
