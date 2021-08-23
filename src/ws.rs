use std::sync::Arc;

use crate::{protocol, Client, Room, Rooms};
use futures::{FutureExt, StreamExt};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    Mutex,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Error,
};

pub async fn client_connection(ws: WebSocket, rooms: Rooms, room_id: String) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().to_simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender.clone()),
    };

    let client_room = rooms.lock().await.get_mut(&room_id).unwrap().clone();
    client_room.lock().await.clients.push(new_client);
    let _ = client_sender
        .clone()
        .send(Ok(Message::text(format!("[{}] Connected", room_id))));

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => {
                println!("received message from {}", uuid);
                msg
            }
            Err(e) => {
                println!("error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        let message = match msg.to_str() {
            Ok(v) => v,
            Err(_) => return,
        };

        let result = protocol::receive(&uuid, &client_room, message);
        match result {
            Ok(message) => {
                broadcast(
                    &*uuid,
                    &*serde_json::to_string(&message).unwrap(),
                    &client_room,
                )
                .await;
            }
            Err(err) => send(&client_sender, &*serde_json::to_string(&err).unwrap()).await,
        }
    }

    {
        let room_clients = &mut client_room.lock().await.clients;
        let index = room_clients
            .iter()
            .position(|c| c.client_id == uuid)
            .unwrap();
        room_clients.remove(index);
    }

    println!("{} disconnected", uuid);
}

async fn send(client_sender: &UnboundedSender<Result<Message, Error>>, message: &str) {
    let _ = client_sender.send(Ok(Message::text(message)));
}

async fn broadcast(client_id: &str, message: &str, client_room: &Arc<Mutex<Room>>) {
    let client_room = client_room.lock().await;
    for client in &client_room.clients {
        if client.client_id != client_id {
            if let Some(sender) = &client.sender {
                println!("Room: {}, Broadcasting: {}", client_room.room_id, message);
                let _ = sender.send(Ok(Message::text(format!(
                    "[{}] {}",
                    client_room.room_id, message
                ))));
            }
        }
    }
}
