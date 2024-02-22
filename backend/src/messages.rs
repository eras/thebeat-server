use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) type Id = uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct HR {
    pub hr: i32,
    pub time_utc: chrono::DateTime<chrono::Utc>,
    pub audio_file: String,
}

impl From<PutHR> for HR {
    fn from(put_hr: PutHR) -> Self {
        HR {
            hr: put_hr.hr,
            time_utc: chrono::Utc::now(),
            audio_file: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PutHR {
    pub hr: i32,
    pub volume: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PutVolume {
    pub volume: f64,
    pub volume_changer_uuid: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PutVolumeResponse {
    pub volume: f64,
    pub volume_change_index: u64,
    pub volume_changer_uuid: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct AllHR {
    pub data: HashMap<Id, HR>,
    pub volume_change_index: u64,
    pub volume: f64,
    pub volume_changer_uuid: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct HRPutResponse {
    pub volume_change_index: u64,
    pub volume_changer_uuid: uuid::Uuid,
    pub volume: f64,
}
