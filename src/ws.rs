use crate::{Client, Rooms};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

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

    rooms
        .lock()
        .await
        .get_mut(&room_id)
        .unwrap()
        .clients
        .push(new_client);
    let _ = client_sender.send(Ok(Message::text(format!("[{}] Connected", room_id))));

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, msg, &rooms).await;
    }

    {
        let mut rooms_lock = rooms.lock().await;
        let room_clients = &mut rooms_lock.get_mut(&room_id).unwrap().clients;
        let index = room_clients
            .iter()
            .position(|c| c.client_id == uuid)
            .unwrap();
        room_clients.remove(index);
    }

    println!("{} disconnected", uuid);
}

async fn client_msg(client_id: &str, msg: Message, rooms: &Rooms) {
    println!("received message from {}: {:?}", client_id, msg);

    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let rooms_lock = rooms.lock().await;
    let room_of_client = &rooms_lock
        .iter()
        .find(|(_, room)| room.clients.iter().any(|c| c.client_id == client_id))
        .unwrap()
        .1;

    for client in &room_of_client.clients {
        if client.client_id != client_id {
            if let Some(sender) = &client.sender {
                println!(
                    "Room: {}, Broadcasting: {}",
                    room_of_client.room_id, message
                );
                let _ = sender.send(Ok(Message::text(format!(
                    "[{}] {}",
                    room_of_client.room_id, message
                ))));
            }
        }
    }
    return;
}
