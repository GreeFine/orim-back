use protocol::Object;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{ws::Message, Filter, Rejection};

use crate::rejection::handle_rejection;

mod handlers;
mod protocol;
mod rejection;
mod ws;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug)]
pub struct Room {
    pub room_id: String,
    pub name: String,
    pub clients: Vec<Client>,
    pub objects: Vec<Object>,
}

type Rooms = Arc<Mutex<HashMap<String, Arc<Mutex<Room>>>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    println!("Configuring websocket route");
    let index = warp::path::end().map(|| "ORIM API");

    let join_route = warp::path!("join" / String)
        .and(warp::ws())
        .and(with_rooms(rooms.clone()))
        .and_then(handlers::join_handler);
    let new_route = warp::path("new")
        .and(warp::ws())
        .and(with_rooms(rooms.clone()))
        .and_then(handlers::new_handler);

    let routes = warp::get()
        .and(index.or(new_route).or(join_route))
        .recover(handle_rejection)
        .with(warp::cors().allow_any_origin());
    println!("Starting server http://localhost:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_rooms(rooms: Rooms) -> impl Filter<Extract = (Rooms,), Error = Infallible> + Clone {
    warp::any().map(move || rooms.clone())
}
