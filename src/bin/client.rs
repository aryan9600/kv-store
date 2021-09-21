use clap::{App, Arg, SubCommand};
use kv_store::{KVStore, KVStoreError, Result};
use std::process::exit;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").required(true))
                .arg(Arg::with_name("val").required(true))
        )
        .subcommand(
            SubCommand::with_name("get")
                .arg(Arg::with_name("key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .arg(Arg::with_name("key").required(true)),
        )
        .get_matches();

    let log_path = "kvs.log";
    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("key").unwrap();
            let val = matches.value_of("val").unwrap();

            let mut store = KVStore::open(log_path)?;
            if let Some(val) = store.set(key.to_string(), val.to_string())? {
                println!("{}", val);
            }
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("key").unwrap();

            let mut store = KVStore::open(log_path)?;
            if let Some(value) = store.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("key").unwrap();

            let mut store = KVStore::open(log_path)?;
            match store.rm(key.to_string()) {
                Ok(()) => {}
                Err(KVStoreError::KeyNotFound(key)) => {
                    println!("Key {} not found", key);
                    exit(1)
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
