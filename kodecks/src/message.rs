use crate::{action::AvailableAction, id::ObjectId, variable::VariableList};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageDialog {
    pub messages: Vec<MessageBox>,
    pub allowed_action: Option<AvailableAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub variables: VariableList,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageBox {
    pub message: Message,
    pub position: MessageBoxPosition,
    pub pointers: Vec<Pointer>,
    pub custom_trigger: Option<String>,
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageBoxPosition {
    #[default]
    Auto,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pointer {
    pub target: PointerTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PointerTarget {
    Card { id: ObjectId },
    PlayersLife,
    OpponentsLife,
}
