use std::collections::HashMap;

use kv_store::{KVStore, KVStoreError, models::{GetBody, RmBody, RmItem, SetItem, SetBody}};
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
    let store = KVStore::open(log_path).unwrap();
    rocket::build()
        .mount("/", routes![index, set, get, rm])
        .manage(store)
}

#[get("/")]
fn index(_state: &State<KVStore>) -> Json<HashMap<String, bool>> {
    let mut response = HashMap::new();
    response.insert("up".into(), true);
    Json(response)
}

#[post("/set", format = "json", data = "<item>")]
fn set(state: &State<KVStore>, item: Json<SetItem>) -> Result<Json<SetBody>> {
    let store= state.inner();
    let val = store.set(item.key.clone(), item.val.clone())?;
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
fn get(state: &State<KVStore>, key: String) -> Result<Json<GetBody>> {
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
fn rm(state: &State<KVStore>, item: Json<RmItem>) -> Result<Json<RmBody>> {
    let store = state.inner();
    let val = store.rm(item.key.clone());
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
