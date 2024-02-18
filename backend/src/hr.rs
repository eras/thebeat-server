use crate::error::Error;
use crate::expiring::Expiring;
use crate::messages;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use std::sync::Arc;
use tokio::sync::Mutex;

type RoomLabel = String;

pub(crate) struct DatabaseContent {
    by_room: Expiring<RoomLabel, Expiring<messages::Id, messages::HR>>,
}

const EXPIRATION_TIME: chrono::Duration = chrono::Duration::seconds(10);

impl DatabaseContent {
    pub fn new() -> DatabaseContent {
        DatabaseContent {
            by_room: Expiring::new(EXPIRATION_TIME),
        }
    }

    pub fn refresh(&mut self, room_name: &str) {
        self.by_room.refresh(&String::from(room_name));
    }

    pub fn get_mut(&mut self, room_name: &str) -> &mut Expiring<messages::Id, messages::HR> {
        self.by_room
            .get_or_put_mut(String::from(room_name), || Expiring::new(EXPIRATION_TIME))
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

    room.purge_old_entries();

    let retval = Ok(Json(messages::AllHR {
        data: (*room).all(),
    }));

    content.purge_old_entries();
    retval
}

#[route(PUT, uri = "/hr/<room_name>/<uuid>", format = "json", data = "<hr>")]
pub(crate) async fn put_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
    hr: Json<messages::PutHR>,
) -> Result<Json<()>, Error> {
    let mut content = db.content.lock().await;

    let room = content.get_mut(&room_name);
    room.put(uuid, hr.0.into());
    content.refresh(&room_name);

    content.purge_old_entries();
    Ok(Json(()))
}

#[route(DELETE, uri = "/hr/<room_name>/<uuid>")]
pub(crate) async fn del_hr(
    db: &State<Database>,
    room_name: String,
    uuid: uuid::Uuid,
) -> Result<Json<()>, Error> {
    let mut content = db.content.lock().await;
    let room = content.get_mut(&room_name);

    room.remove(&uuid);
    if room.is_empty() {
        content.remove(&room_name)
    }

    content.purge_old_entries();
    Ok(Json(()))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_hr, put_hr, del_hr]
}
