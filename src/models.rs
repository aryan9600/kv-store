use rocket::serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SetItem {
    pub key: String,
    pub val: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RmItem {
    pub key: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GetBody {
    found: bool,
    val: Option<String>
}

impl From<(bool, Option<String>)> for GetBody {
    fn from(body: (bool, Option<String>)) -> Self {
        GetBody {
            found: body.0,
            val: body.1
        }
    }
}

impl fmt::Display for GetBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(val) = &self.val {
            write!(f, "{{found: {}, val: {}}}", self.found, val)
        } else {
            write!(f, "{{found: {}, val: null}}", self.found)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SetBody {
    inserted: bool,
    ejected_val: Option<String>
}

impl From<(bool, Option<String>)> for SetBody {
    fn from(body: (bool, Option<String>)) -> Self {
        SetBody {
            inserted: body.0,
            ejected_val: body.1
        }
    }
}

impl fmt::Display for SetBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(val) = &self.ejected_val {
            write!(f, "{{inserted: {}, ejected_val: {}}}", self.inserted, val)
        } else {
            write!(f, "{{inserted: {}, ejected_val: null}}", self.inserted)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RmBody {
    removed: bool,
    found: bool,
    ejected_val: Option<String>
}

impl From<(bool, Option<String>)> for RmBody {
    fn from(body: (bool, Option<String>)) -> Self {
        RmBody {
            removed: body.0,
            found: body.0,
            ejected_val: body.1
        }
    }
}

impl fmt::Display for RmBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(val) = &self.ejected_val {
            write!(f, "{{removed: {}, found: {}, ejected_val: {}}}", self.removed, self.found, val)
        } else {
            write!(f, "{{removed: {}, found: {}, ejected_val: null}}", self.removed, self.found)
        }
    }
}
