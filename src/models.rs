use rocket::serde::{Serialize, Deserialize};

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
