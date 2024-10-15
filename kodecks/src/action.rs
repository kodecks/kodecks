use crate::{
    command::ActionCommand,
    env::Environment,
    id::ObjectId,
    message::{Message, MessageDialog},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum AvailableAction {
    SelectCard {
        cards: Vec<ObjectId>,
        score_factor: i32,
    },
    Attack {
        attackers: Vec<ObjectId>,
    },
    Block {
        blockers: Vec<ObjectId>,
    },
    CastCard {
        cards: Vec<ObjectId>,
    },
    EndTurn,
    Continue,
}

impl PartialOrd for AvailableAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AvailableAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let index = |action: &AvailableAction| match action {
            AvailableAction::SelectCard { .. } => 0,
            AvailableAction::Attack { .. } => 1,
            AvailableAction::Block { .. } => 2,
            AvailableAction::CastCard { .. } => 3,
            AvailableAction::EndTurn => 4,
            AvailableAction::Continue => 5,
        };
        index(self).cmp(&index(other))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PlayerAvailableActions {
    pub player: u8,
    pub actions: AvailableActionList,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_dialog: Option<MessageDialog>,
}

impl PlayerAvailableActions {
    pub fn new(player: u8) -> Self {
        Self {
            player,
            actions: AvailableActionList::new(),
            instructions: None,
            message_dialog: None,
        }
    }

    pub fn validate(&self, player: u8, action: &Action) -> bool {
        if player != self.player {
            return false;
        }
        self.actions.validate(action)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
#[serde(transparent)]
pub struct AvailableActionList(Vec<AvailableAction>);

impl AvailableActionList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AvailableAction> {
        self.0.iter()
    }

    pub fn validate(&self, action: &Action) -> bool {
        match action {
            Action::CastCard { card } => {
                self.0
                    .iter()
                    .any(|action| matches!(action, AvailableAction::CastCard { cards } if cards.contains(card)))
            }
            Action::SelectCard { card } => {
                self.0
                    .iter()
                    .any(|action| matches!(action, AvailableAction::SelectCard { cards, .. } if cards.contains(card)))
            }
            Action::Attack { attackers } => {
                self.0
                    .iter()
                    .any(|action| matches!(action, AvailableAction::Attack { attackers: available } if attackers.iter().all(|a| available.contains(a))))
            }
            Action::Block { pairs } => {
                self.0
                    .iter()
                    .any(|action| matches!(action, AvailableAction::Block { blockers } if pairs.iter().all(|(_, b)| blockers.contains(b))))
            }
            Action::EndTurn => self.0.iter().any(|action| matches!(action, AvailableAction::EndTurn)),
            Action::Continue => self.0.iter().any(|action| matches!(action, AvailableAction::Continue)),
            _ => true,
        }
    }

    pub fn can_end_turn(&self) -> bool {
        self.iter()
            .any(|action| matches!(action, AvailableAction::EndTurn))
    }

    pub fn attackers(&self) -> Vec<ObjectId> {
        self.iter()
            .filter_map(|action| {
                if let AvailableAction::Attack { attackers } = action {
                    Some(attackers)
                } else {
                    None
                }
            })
            .flatten()
            .copied()
            .collect()
    }

    pub fn blockers(&self) -> Vec<ObjectId> {
        self.iter()
            .filter_map(|action| {
                if let AvailableAction::Block { blockers } = action {
                    Some(blockers)
                } else {
                    None
                }
            })
            .flatten()
            .copied()
            .collect()
    }

    pub fn castable_cards(&self) -> Vec<ObjectId> {
        self.iter()
            .flat_map(|action| {
                if let AvailableAction::CastCard { cards } = action {
                    cards
                } else {
                    &[][..]
                }
            })
            .copied()
            .collect()
    }

    pub fn selectable_cards(&self) -> Vec<ObjectId> {
        self.iter()
            .flat_map(|action| {
                if let AvailableAction::SelectCard { cards, .. } = action {
                    cards
                } else {
                    &[][..]
                }
            })
            .copied()
            .collect()
    }

    pub fn can_continue(&self) -> bool {
        self.iter()
            .any(|action| matches!(action, AvailableAction::Continue))
    }

    pub fn default_action(&self, env: &Environment) -> Option<Action> {
        for action in self.iter() {
            match action {
                AvailableAction::SelectCard { cards, .. } => {
                    let oldest_card = cards
                        .iter()
                        .filter_map(|card| env.state.find_card(*card).ok())
                        .min_by_key(|card| card.timestamp())
                        .map(|card| card.id());
                    if let Some(card) = oldest_card {
                        return Some(Action::SelectCard { card });
                    }
                }
                AvailableAction::CastCard { .. } => {
                    continue;
                }
                AvailableAction::Attack { .. } => {
                    return Some(Action::Attack { attackers: vec![] });
                }
                AvailableAction::Block { .. } => {
                    return Some(Action::Block { pairs: vec![] });
                }
                AvailableAction::EndTurn => return Some(Action::EndTurn),
                AvailableAction::Continue => return Some(Action::Continue),
            }
        }
        None
    }
}

impl AsRef<[AvailableAction]> for AvailableActionList {
    fn as_ref(&self) -> &[AvailableAction] {
        &self.0
    }
}

impl FromIterator<AvailableAction> for AvailableActionList {
    fn from_iter<I: IntoIterator<Item = AvailableAction>>(iter: I) -> Self {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        vec.sort();
        Self(vec)
    }
}

impl IntoIterator for AvailableActionList {
    type Item = AvailableAction;
    type IntoIter = std::vec::IntoIter<AvailableAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Action {
    CastCard { card: ObjectId },
    SelectCard { card: ObjectId },
    Attack { attackers: Vec<ObjectId> },
    Block { pairs: Vec<(ObjectId, ObjectId)> },
    EndTurn,
    Concede,
    Continue,
    DebugCommand { commands: Vec<ActionCommand> },
}
