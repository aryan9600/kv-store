use nats::{Connection, Subscription};
use rocket::serde::{Serialize, Deserialize};
use log::error;

use crate::Result;

/// Publish an action to a subject in the NATS server.
pub fn publish_action<'de, T>(conn: &Connection, subject: &str, action: Box<T>) -> Result<()>
    where T: Serialize + Deserialize<'de> 
{
    conn.publish(
        subject, serde_json::to_vec(&action)?
    )?;
    Ok(())
}

/// Subscribe to a subject in the NATS server.
pub fn subscribe(conn: &Connection, subject: &str) -> Result<Subscription> {
    let sub = conn.subscribe(subject)?;
    Ok(sub)
}

/// Try to connect to a NATS server. Return None if the call to connect errors.
pub fn connect(host: String) -> Option<Connection> {
    match nats::connect(&host) {
        Ok(conn) => Some(conn),
        Err(err) => { 
            error!("Could not connect to nats server: {}", &err);
            None
        }
    }
}
