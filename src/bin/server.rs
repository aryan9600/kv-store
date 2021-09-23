use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use kv_store::{
    models::{GetBody, RmBody, RmItem, SetBody, SetItem},
    pubsub, ConnStrings, KVStore, KVStoreError,
};
use nats::Connection;
use rocket::serde::json::Json;
use rocket::{http::hyper::Uri, response::status, Config, State};

#[macro_use]
extern crate rocket;

type Result<T, E = rocket::response::Debug<KVStoreError>> = std::result::Result<T, E>;

#[launch]
fn rocket() -> _ {
    let conn_strings = ConnStrings::load();
    let nc = pubsub::connect(conn_strings.nats_host());
    let store = KVStore::open(conn_strings.log_file_path()).expect("Could not open KVStore");

    let server_host = conn_strings
        .server_host()
        .parse::<Uri>()
        .expect("Invalid server url, please validate the value of KVSTORE_HOST, if it's set");
    let host = server_host.host().unwrap();
    let port = server_host.port_u16().unwrap();

    let config = Config {
        address: IpAddr::V4(Ipv4Addr::from_str(host).unwrap()),
        port,
        ..Config::default()
    };

    rocket::build()
        .mount("/", routes![index, set, get, rm])
        .configure(&config)
        .manage(store)
        .manage(nc)
}

#[get("/")]
fn index(
    _store_state: &State<KVStore>,
    _conn_state: &State<Option<Connection>>,
) -> Json<HashMap<String, bool>> {
    let mut response = HashMap::new();
    response.insert("up".into(), true);
    Json(response)
}

#[post("/set", format = "json", data = "<item>")]
fn set(
    store_state: &State<KVStore>,
    conn_state: &State<Option<Connection>>,
    item: Json<SetItem>,
) -> Result<status::Created<Json<SetBody>>> {
    let store = store_state.inner();
    let val = store.set(item.key.clone(), item.val.clone())?;
    let conn = conn_state.inner();
    if let Some(nc) = conn {
        pubsub::publish_action(nc, "set", Box::new(item.into_inner()))?;
    }
    let response = status::Created::new("");
    if let Some(val) = val {
        return Ok(response.body(Json(SetBody::from((true, Some(val))))));
    } else {
        return Ok(response.body(Json(SetBody::from((true, None)))));
    }
}

#[get("/get?<key>")]
fn get(
    state: &State<KVStore>,
    _conn_state: &State<Option<Connection>>,
    key: String,
) -> Result<Json<GetBody>> {
    let store = state.inner();
    let val = store.get(key.clone());
    if let Ok(Some(val)) = val {
        return Ok(Json(GetBody::from((true, Some(val)))));
    } else {
        return Ok(Json(GetBody::from((false, None))));
    }
}

#[delete("/rm", format = "json", data = "<item>")]
fn rm(
    store_state: &State<KVStore>,
    conn_state: &State<Option<Connection>>,
    item: Json<RmItem>,
) -> Result<Json<RmBody>> {
    let store = store_state.inner();
    let val = store.rm(item.key.clone());
    let conn = conn_state.inner();
    if let Some(nc) = conn {
        pubsub::publish_action(nc, "rm", Box::new(item.into_inner()))?;
    }
    if let Ok(Some(val)) = val {
        return Ok(Json(RmBody::from((true, Some(val)))));
    } else {
        return Ok(Json(RmBody::from((false, None))));
    }
}
