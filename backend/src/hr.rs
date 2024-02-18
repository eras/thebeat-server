use crate::error::Error;
use crate::messages;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;

struct DbData {
    data: messages::HR,
    insert_count: u64,
    // double bookkeeping. Just thinking of db keeping separate track of this, in case
    // I want to reuse this code for something else.. and maybe the time in the data provided
    // to the client isn't very useful? they can detect dropped clients from the lack of data.
    insert_time: chrono::DateTime<chrono::Utc>,
}

pub(crate) struct DatabaseContentForRoom {
    data: HashMap<messages::Id, DbData>,
    by_insert: BTreeMap<u64, messages::Id>,
    insert_count: u64,
}

impl DatabaseContentForRoom {
    fn new() -> Self {
        DatabaseContentForRoom {
            data: HashMap::new(),
            by_insert: BTreeMap::new(),
            insert_count: 0u64,
        }
    }

    fn put(&mut self, id: messages::Id, data: messages::HR) {
        self.remove(&id.clone());
        self.data.insert(
            id,
            DbData {
                data: data,
                insert_count: self.insert_count,
                insert_time: chrono::Utc::now(),
            },
        );
        self.by_insert.insert(self.insert_count, id);
        self.insert_count += 1
    }

    fn remove(&mut self, id: &messages::Id) {
        match self.data.get(id) {
            None => (),
            Some(data) => {
                self.by_insert.remove(&data.insert_count);
                self.data.remove(id);
            }
        }
    }

    fn purge_old_entries(&mut self) {
        let deadline = chrono::Utc::now() - chrono::Duration::seconds(10);
        loop {
            match self.by_insert.first_key_value() {
                None => break,
                Some((_, id)) if self.data.get(id).unwrap().insert_time < deadline => {
                    self.remove(&id.clone());
                }
                Some(_) => break,
            }
        }
    }

    fn all(&self) -> HashMap<messages::Id, messages::HR> {
        self.data
            .iter()
            .map(|(k, v)| (*k, v.data.clone()))
            .collect()
    }
}

type RoomLabel = String;

pub(crate) struct DatabaseContent {
    by_room: HashMap<RoomLabel, DatabaseContentForRoom>,
}

impl DatabaseContent {
    pub fn new() -> DatabaseContent {
        DatabaseContent {
            by_room: HashMap::new(),
        }
    }

    pub fn get(&mut self, room_name: String) -> &mut DatabaseContentForRoom {
        return self
            .by_room
            .entry(room_name)
            .or_insert_with(|| DatabaseContentForRoom::new());
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
    let room = content.get(room_name);

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
    let room = content.get(room_name);
    room.put(uuid, hr.0.into());
    Ok(Json(()))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_hr, put_hr]
}
