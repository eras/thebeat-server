use crate::error::Error;
use crate::expiring::Expiring;
use crate::messages;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type RoomLabel = String;

#[derive(Clone, Debug)]
pub(crate) struct RoomContent {
    hr: Expiring<messages::Id, messages::HR>,
    volume_changer_uuid: uuid::Uuid,
    volume_change_index: u64,
    volume: f64,
}

impl RoomContent {
    fn new() -> Self {
        RoomContent {
            hr: Expiring::new(EXPIRATION_TIME),
            volume_changer_uuid: uuid::Uuid::new_v4(),
            volume: -10.0,
            volume_change_index: 0u64,
        }
    }
}

pub(crate) struct DatabaseContent {
    by_room: Expiring<RoomLabel, RoomContent>,
}

const AUDIO_FILES: &[&str] = &[
    "heart-beat.wav",
    "beep.wav",
    "heart-beat500.wav",
    "heart-beat1000.wav",
    "heart-beat1500.wav",
    // "beep-100.wav",
    // "beep-200.wav",
    // "beep-300.wav",
    // "beep-400.wav",
];

const EXPIRATION_TIME: chrono::Duration = chrono::Duration::seconds(10);

impl DatabaseContent {
    pub fn new() -> DatabaseContent {
        DatabaseContent {
            by_room: Expiring::new(EXPIRATION_TIME),
        }
    }

    pub fn refresh(&mut self, room_name: &str) {
        self.by_room.refresh(String::from(room_name));
    }

    pub fn get_mut(&mut self, room_name: &str) -> &mut RoomContent {
        self.by_room
            .get_or_put_mut(String::from(room_name), || RoomContent::new())
            .1
    }

    pub fn purge_old_entries(&mut self) {
        self.by_room.purge_old_entries()
    }

    pub fn remove(&mut self, room_name: &str) {
        self.by_room.remove(&String::from(room_name));
    }
}

#[derive(Clone)]
pub(crate) struct Database {
    content: Arc<Mutex<DatabaseContent>>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            content: Arc::new(Mutex::new(DatabaseContent::new())),
        }
    }
}

#[route(GET, uri = "/hr/<room_name>")]
pub(crate) async fn get_hr(
    db: &State<Database>,
    room_name: String,
) -> Result<Json<messages::AllHR>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get_mut(&room_name);

    room.hr.purge_old_entries();

    let retval = Ok(Json(messages::AllHR {
        data: (*room).hr.all(),
        volume_changer_uuid: room.volume_changer_uuid.clone(),
        volume_change_index: room.volume_change_index,
        volume: room.volume,
    }));

    content.purge_old_entries();
    retval
}

fn least_used_audio_in_room(room: &Expiring<messages::Id, messages::HR>) -> Option<String> {
    let mut counts = HashMap::new();
    for audio_file in AUDIO_FILES {
        counts.insert(audio_file.to_string(), 0u32);
    }
    use std::collections::hash_map::Entry;
    for (_id, data) in room.all_ref() {
        match counts.entry(data.audio_file.clone()) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
            }
            Entry::Vacant(_) => (),
        }
    }

    if counts.len() == 0 {
        None
    } else {
        // Find the entry in counts with smallest count
        let mut smallest_count = u32::MAX;
        let mut smallest_key = String::new();
        for (key, count) in counts {
            if count < smallest_count {
                smallest_count = count;
                smallest_key = key.clone();
            }
        }
        Some(smallest_key)
    }
}

#[route(
    POST,
    uri = "/volume/<room_name>",
    format = "json",
    data = "<put_volume>"
)]
pub(crate) async fn put_volume(
    db: &State<Database>,
    room_name: String,
    put_volume: Json<messages::PutVolume>,
) -> Result<Json<messages::PutVolumeResponse>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get_mut(&room_name);

    room.volume = put_volume.0.volume;
    room.volume_changer_uuid = put_volume.0.volume_changer_uuid;
    room.volume_change_index += 1;

    let result = Ok(Json(messages::PutVolumeResponse {
        volume: room.volume,
        volume_changer_uuid: room.volume_changer_uuid.clone(),
        volume_change_index: room.volume_change_index,
    }));
    content.refresh(&room_name);
    content.purge_old_entries();

    result
}

#[route(PUT, uri = "/hr/<room_name>/<uuid>", format = "json", data = "<hr>")]
pub(crate) async fn put_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
    hr: Json<messages::PutHR>,
) -> Result<Json<messages::HRPutResponse>, Error> {
    let mut content = db.content.lock().await;

    let room = content.get_mut(&room_name);
    if let Some(volume) = &hr.volume {
        room.volume = *volume;
        room.volume_change_index += 1;
        room.volume_changer_uuid = uuid.clone();
    }
    let least_used_audio = least_used_audio_in_room(&room.hr);
    let mut db_hr: messages::HR = hr.0.into();
    match room.hr.get(&uuid) {
        None => match least_used_audio {
            Some(least_used_audio) => db_hr.audio_file = least_used_audio,
            None => (),
        },
        Some(old_hr) => {
            db_hr.audio_file = old_hr.audio_file.clone();
        }
    }
    room.hr.put(uuid, db_hr);
    room.hr.purge_old_entries();

    let result = Ok(Json(messages::HRPutResponse {
        volume: room.volume,
        volume_changer_uuid: room.volume_changer_uuid.clone(),
        volume_change_index: room.volume_change_index,
    }));
    content.refresh(&room_name);
    content.purge_old_entries();
    result
}

#[route(DELETE, uri = "/hr/<room_name>/<uuid>")]
pub(crate) async fn del_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
) -> Result<Json<()>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get_mut(&room_name);

    room.hr.remove(&uuid);
    if room.hr.is_empty() {
        content.remove(&room_name)
    }

    content.purge_old_entries();
    Ok(Json(()))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_hr, put_hr, del_hr, put_volume]
}
