use clap::{App, Arg, SubCommand};
use kv_store::{ConnStrings, models::{GetBody, RmBody, RmItem, SetBody, SetItem}, pubsub};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .after_help("Set the KVSTORE_HOST env variable to specify the server host, default: 127.0.0.1:8000")
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the key value pair.")
                .arg(Arg::with_name("key").required(true))
                .arg(Arg::with_name("val").required(true)),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the value of this key.")
                .arg(Arg::with_name("key").required(true))
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove the key from the store.")
                .arg(Arg::with_name("key").required(true))
        )
        .subcommand(
            SubCommand::with_name("sub")
                .about("Subscribe to changes to any of the keys.")
        )
        .get_matches();

    let conn_strings = ConnStrings::load();
    let server_host = conn_strings.server_host();
    let client = reqwest::Client::new();

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("key").expect("Key not provided").to_string();
            let val = matches.value_of("val").expect("Value not provided").to_string();
            let body = SetItem {
                key,
                val
            };

            let resp: SetBody = client.post(format!("{}/set", server_host))
                .json(&body)
                .send()
                .await?
                .json()
                .await?;

            println!("{}", resp);
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("key").expect("Key not provided");

            let resp: GetBody = client.get(format!("{}/get?key={}", server_host, key))
                .send()
                .await?
                .json()
                .await?;
            println!("{}", resp);
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("key").expect("Key not provided").to_string();
            let body = RmItem {
                key
            };

            let resp: RmBody = client.delete(format!("{}/rm", server_host))
                .json(&body)
                .send()
                .await?
                .json()
                .await?;
            println!("{}", resp);
        }
        ("sub", Some(_)) => {
            let conn = pubsub::connect(conn_strings.nats_host());
            if let Some(nc) = conn {
                let rm_sub = pubsub::subscribe(&nc, "rm")?;
                tokio::spawn(async move {
                    for msg in rm_sub.messages() {
                        let rm_item: Option<RmItem> = serde_json::from_slice(&msg.data).ok();
                        if let Some(item) = rm_item {
                            println!("Remove: {}", item);
                        }
                    }
                });
                let set_sub = pubsub::subscribe(&nc, "set")?;
                for msg in set_sub.messages() {
                    let set_item: SetItem = serde_json::from_slice(&msg.data)?;
                    println!("Set: {}", set_item);
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
