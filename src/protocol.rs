use std::sync::Arc;

use crate::Room;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

type ObjectId = u32;

#[derive(Debug, Clone)]
pub struct Object {
    id: ObjectId,
    owner: String,
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum ActionType {
    Update,
    Delete,
    Lock,
    Unlock,
    Broadcast,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectUpdate {
    r#type: ActionType,
    reference: Vec<ObjectId>,
    states: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MessageJSON {
    version: u32,
    object_update: Vec<ObjectUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ErrorWS<'a> {
    pub name: &'a str,
    pub status_code: u32,
    pub message: String,
}

fn action_delete<'a>(
    object_update: &ObjectUpdate,
    room: &Arc<Mutex<Room>>,
) -> Result<(), ErrorWS<'a>> {
    Ok(())
}

pub fn process_actions<'a>(
    message_json: MessageJSON,
    room: &Arc<Mutex<Room>>,
) -> Result<Vec<ObjectUpdate>, ErrorWS<'a>> {
    for object_update in &message_json.object_update {
        match object_update.r#type {
            ActionType::Broadcast => Ok(()),
            ActionType::Delete => action_delete(object_update, room),
            ActionType::Update => todo!(),
            ActionType::Lock => todo!(),
            ActionType::Unlock => todo!(),
            _ => Err(ErrorWS {
                name: "Unkown action",
                status_code: 400,
                message: String::from("Action for object update is unkown"),
            }),
        };
    }
    // FIXME: Do we realy want this
    Ok(message_json.object_update)
}

pub fn receive<'a>(
    client_id: &str,
    client_room: &Arc<Mutex<Room>>,
    message: &str,
) -> Result<Vec<ObjectUpdate>, ErrorWS<'a>> {
    let message_json: std::result::Result<MessageJSON, _> = serde_json::from_str(message);

    match message_json {
        Ok(message_json) => {
            println!("Message: {:#?}", message_json);
            Ok(process_actions(message_json, client_room)?)
        }
        Err(error) => {
            println!("Error: {}", error);
            Err(ErrorWS {
                name: "JSON Deserialize",
                status_code: 400,
                message: format!("Unable to parse received JSON\n{}", error),
            })
        }
    }
}
