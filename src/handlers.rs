use crate::{rejection::RoomNotFound, ws, Result, Room, Rooms};
use names::{Generator, Name};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Reply;

pub async fn join_handler(room_id: String, ws: warp::ws::Ws, rooms: Rooms) -> Result<impl Reply> {
    let room_exist = rooms.lock().await.get(&room_id).is_some();
    if room_exist {
        Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, rooms, room_id)))
    } else {
        Err(warp::reject::custom(RoomNotFound))
    }
}

pub async fn new_handler(ws: warp::ws::Ws, rooms: Rooms) -> Result<impl Reply> {
    let mut generator = Generator::with_naming(Name::Numbered);
    let room_id = generator.next().unwrap();
    let room: Arc<Mutex<Room>> = Arc::new(Mutex::new(Room {
        room_id: room_id.clone(),
        name: String::from("New Project"),
        clients: Vec::new(),
        objects: Vec::new(),
    }));

    rooms.lock().await.insert(room_id.clone(), room);
    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, rooms, room_id)))
}
