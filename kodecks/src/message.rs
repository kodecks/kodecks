use crate::{action::AvailableAction, id::ObjectId, variable::VariableList};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct MessageDialog {
    pub messages: Vec<MessageBox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_action: Option<AvailableAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct Message {
    pub id: String,
    #[serde(default)]
    pub variables: VariableList,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct MessageBox {
    pub message: Message,
    pub position: MessageBoxPosition,
    pub pointers: Vec<Pointer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_trigger: Option<String>,
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum MessageBoxPosition {
    #[default]
    Auto,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub struct Pointer {
    pub target: PointerTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PointerTarget {
    Card { id: ObjectId },
    PlayersLife,
    OpponentsLife,
}
