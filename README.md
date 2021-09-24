# kv-store

A key-value store exposed via a web server. A CLI client is also provided which consumes the web service.

## Architecture
<img src="https://raw.githubusercontent.com/aryan9600/kv-store/main/kv-store-arch.png" alt="KV Store Architecture"/>

## Usage

#### Installing
* Clone this repository.
* Install [Rust](https://www.rust-lang.org/).

#### CLI Operations
* get(key): `cargo run --bin client -- get {key}`, get the value of the key, if present.
* set(key, val): `cargo run --bin client -- set {key} {val}`, set the key-value pair
* rm(key): `cargo run --bin client -- rm {key}` remove the key, if present.
* sub: `cargo run --bin client -- sub` subscribe to any changes happening to any keys.

#### Server API
Run the server: `cargo run --bin server`
| Route     | Body                                               | Response                                                                         | Status |
|-----------|----------------------------------------------------|----------------------------------------------------------------------------------|--------|
| /set      | ```{     "key": "abc",     "val": "xyz" }``` | ```{     "inserted": true,     "ejected_val": null } ```                    | 201    |
| /get?key=abc |                                                    | ```{     "found": true,     "inserted_val": "xyz" }```                     | 200    |
| /rm       | ```{     "key": "abc" }```                   | ```{     "found": true,     "removed": true,     "ejected_val": "xyz" }``` | 200    |

#### Docker
* Build the server image: `docker build -t kv-store .`
* Run the container: `docker run -p 8000:8000 --env KVSTORE_SERVER_HOST=0.0.0.0:8000 kv-store`
To subscribe to changes happening to keys, we also need to run a NATS server, which serves the purpose of a message queue. There's a `docker-compose.yml` provided to make this easier.
* Populate the `.env`
```
KVSTORE_NATS_HOST=nats:4222
KVSTORE_SERVER_HOST=0.0.0.0:8000
```
* Run the services: `docker-compose -f docker-comppose.dev.yml up --build`

#### Tests
* To run tests: `cargo test`


## Repo Structure
* `src/store.rs`: Contains the main buisness logic behing the get, set and rm operations.
* `src/error.rs`: Defines the custom error/result types.
* `src/models.rs`: Contains the various server request/response structures.
* `src/pubsub.rs`: Contains helper methods related to publishing and subscribing to NATS.
* `src/bin/client.rs`: Defines the CLI which consumes the web service and/or subscribes to changes to keys.
* `src/bin/server.rs`: Launches the server and publishes any changes happening to any keys.


## CI/CD
GitHub Actions is used for CI. `cargo fmt` and `cargo clippy` is used to ensure linting and code quality. Tests are run and code coverage is reported using [tarpaulin](https://github.com/xd009642/tarpaulin) and [coveralls](https://coveralls.io). The production server is deployed on an EC2 instance(http://13.233.94.11) using `docker-compose`, which runs three services:
* the kv-store server
* a NATS server 
* a HAProxy load balancer.

A Jenkins server is also hosted on the same instance, which is used for deploying new changes. A push to the `main` branch will trigger a Jenkins job, which checks out the latest code, zips it and pushes it to a S3 bucket. AWS CodeDeploy deployment is then triggered which runs `deploy.sh`, a shell script which performs rolling deployments.
