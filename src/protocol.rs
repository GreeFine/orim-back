use std::sync::Arc;

use crate::Room;
use serde::Deserialize;
use tokio::sync::Mutex;

type ObjectId = u32;

#[derive(Debug, Clone)]
pub struct Object {
    id: ObjectId,
    owner: String,
    state: String,
}

#[derive(Debug, Deserialize)]
enum ActionType {
    Lock,
    Unlock,
    Update,
    Delete,
}

#[derive(Debug, Deserialize)]
struct ObjectUpdate {
    r#type: ActionType,
    reference: Vec<ObjectId>,
    states: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct MessageJSON {
    version: String,
    object_update: Vec<ObjectUpdate>,
}

struct ErrorWS<'a> {
    name: &'a str,
    status_code: u32,
    message: &'a str,
}

pub fn process_actions<'a>(
    message_json: MessageJSON,
    room: &Arc<Mutex<Room>>,
) -> Result<Vec<ObjectUpdate>, ErrorWS<'a>> {
    for object_update in message_json.object_update {
        match object_update.r#type {
            ActionType::Delete => {}
            _ => Err(ErrorWS {
                name: "Unkown action",
                status_code: 400,
                message: "Action for object update is unkown",
            }),
        }
    }
}

pub fn receive<'a>(
    client_id: &str,
    client_room: &Arc<Mutex<Room>>,
    message: &str,
) -> Result<Vec<Object>, ErrorWS<'a>> {
    let message_json: std::result::Result<MessageJSON, _> = serde_json::from_str(message);

    match message_json {
        Ok(message_json) => {
            println!("Message: {:#?}", message_json);
            Ok(process_actions(message_json, client_room))
        }
        Err(error) => {
            println!("Error: {}", error);
            Err(ErrorWS {
                name: "JSON Deserialize",
                status_code: 400,
                message: "Unable to parse receive JSON ",
            })
        }
    }
}
