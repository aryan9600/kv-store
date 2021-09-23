# kv-store
---
A key-value store exposed via a web server. A CLI client is also provided which consumes the web service.

## Usage
---
#### CLI Operations
* get(key): `cargo run --bin client -- get {key}`, get the value of the key, if present.
* set(key, val): `cargo run --bin client -- set {key} {val}`, set the key-value pair
* rm(key): `cargo run --bin client -- rm {key}` remove the key, if present.
* sub: `cargo run --bin client -- sub` subscribe to any changes happening to any keys.

#### Server API
| Route     | Body                                               | Response                                                                         | Status |
|-----------|----------------------------------------------------|----------------------------------------------------------------------------------|--------|
| /set      | ```json {     "key": "abc",     "val": "xyz" } ``` | ```json {     "inserted": true,     "ejected_val": null } ```                    | 201    |
| /get?key=abc |                                                    | ```json {     "found": true,     "inserted_val": "xyz" } ```                     | 200    |
| /rm       | ```json {     "key": "abc" } ```                   | ```json {     "found": true,     "removed": true,     "ejected_val": "xyz" } ``` | 200    |
