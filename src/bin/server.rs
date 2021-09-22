use std::collections::HashMap;

use kv_store::{KVStore, KVStoreError, models::{GetBody, RmBody, RmItem, SetItem, SetBody}, pubsub};
use nats::Connection;
use rocket::State;
use rocket::serde::json::Json;

#[macro_use] extern crate rocket;

type Result<T, E = rocket::response::Debug<KVStoreError>> = std::result::Result<T, E>;

#[launch]
fn rocket() -> _ {
    let mut log_path = String::from("kvs.log");
    // Try to fetch KVSTORE_LOG_PATH from env to customize log file path.
    if let Ok(val) = std::env::var("KVSTORE_LOG_PATH") {
        log_path = val;
    }
    let mut nats_host = String::from("127.0.0.1:4444");
    if let Ok(val) = std::env::var("KVSTORE_NATS_HOST") {
        nats_host = val;
    }
    let nc = pubsub::connect(nats_host);
    let store = KVStore::open(log_path).expect("Could not open KVStore");

    rocket::build()
        .mount("/", routes![index, set, get, rm])
        .manage(store)
        .manage(nc)
}

#[get("/")]
fn index(_store_state: &State<KVStore>, _conn_state: &State<Option<Connection>>) -> Json<HashMap<String, bool>> {
    let mut response = HashMap::new();
    response.insert("up".into(), true);
    Json(response)
}

#[post("/set", format = "json", data = "<item>")]
fn set(store_state: &State<KVStore>, conn_state: &State<Option<Connection>>, item: Json<SetItem>) -> Result<Json<SetBody>> {
    let store = store_state.inner();
    let val = store.set(item.key.clone(), item.val.clone())?;
    let conn = conn_state.inner();
    if let Some(nc) = conn {
        pubsub::publish_action(nc, "set", Box::new(item.into_inner()))?;
    }
    if let Some(val) = val {
        return Ok(Json(
            SetBody::from((true, Some(val)))
        ))
    } else  {
        return Ok(Json(
            SetBody::from((true, None))
        ))
    }
}

#[get("/get?<key>")]
fn get(state: &State<KVStore>, _conn_state: &State<Option<Connection>>, key: String) -> Result<Json<GetBody>> {
    let store = state.inner();
    let val = store.get(key.clone());
    if let Ok(Some(val)) = val {
        return Ok(Json(
            GetBody::from((true, Some(val)))
        ))
    } else {
        return Ok(Json(
            GetBody::from((false, None))
        ))
    }
}

#[delete("/rm", format = "json", data = "<item>")]
fn rm(store_state: &State<KVStore>, conn_state: &State<Option<Connection>>, item: Json<RmItem>) -> Result<Json<RmBody>> {
    let store = store_state.inner();
    let val = store.rm(item.key.clone());
    let conn = conn_state.inner();
    if let Some(nc) = conn {
        pubsub::publish_action(nc, "rm", Box::new(item.into_inner()))?;
    }
    if let Ok(Some(val)) = val {
        return Ok(Json(
            RmBody::from((true, Some(val)))
        ))
    } else {
        return Ok(Json(
            RmBody::from((false, None))
        ))
    }
}
