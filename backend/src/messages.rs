use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) type Id = uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct HR {
    pub hr: i32,
    pub time_utc: chrono::DateTime<chrono::Utc>,
}

impl From<PutHR> for HR {
    fn from(put_hr: PutHR) -> Self {
        HR {
            hr: put_hr.hr,
            time_utc: chrono::Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PutHR {
    pub hr: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct AllHR {
    pub data: HashMap<Id, HR>,
}
