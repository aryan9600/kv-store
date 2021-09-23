use nats::{Connection, Subscription};
use rocket::serde::{Serialize, Deserialize};
use log::error;

use crate::Result;

pub fn publish_action<'de, T>(conn: &Connection, subject: &str, action: Box<T>) -> Result<()>
    where T: Serialize + Deserialize<'de> 
{
    conn.publish(
        subject, serde_json::to_vec(&action)?
    )?;
    Ok(())
}

pub fn subscribe(conn: &Connection, subject: &str) -> Result<Subscription> {
    let sub = conn.subscribe(subject)?;
    Ok(sub)
}

pub fn connect(host: String) -> Option<Connection> {
    match nats::connect(&host) {
        Ok(conn) => Some(conn),
        Err(err) => { 
            error!("Could not connect to nats server: {}", &err);
            None
        }
    }
}
