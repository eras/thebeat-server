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

pub(crate) struct DatabaseContent {
    by_room: HashMap<RoomLabel, Expiring<messages::Id, messages::HR>>,
}

impl DatabaseContent {
    pub fn new() -> DatabaseContent {
        DatabaseContent {
            by_room: HashMap::new(),
        }
    }

    pub fn get(&mut self, room_name: &str) -> &mut Expiring<messages::Id, messages::HR> {
        return self
            .by_room
            .entry(String::from(room_name))
            .or_insert_with(|| Expiring::new(chrono::Duration::seconds(10)));
    }

    pub fn remove(&mut self, room_name: &str) {
        self.by_room.remove(room_name);
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
    let room = content.get(&room_name);

    room.purge_old_entries();

    Ok(Json(messages::AllHR {
        data: (*room).all(),
    }))
}

#[route(PUT, uri = "/hr/<room_name>/<uuid>", format = "json", data = "<hr>")]
pub(crate) async fn put_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
    hr: Json<messages::PutHR>,
) -> Result<Json<()>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get(&room_name);
    room.put(uuid, hr.0.into());
    Ok(Json(()))
}

#[route(DELETE, uri = "/hr/<room_name>/<uuid>")]
pub(crate) async fn del_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
) -> Result<Json<()>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get(&room_name);
    room.remove(&uuid);
    if room.is_empty() {
        content.remove(&room_name)
    }
    Ok(Json(()))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_hr, put_hr, del_hr]
}
