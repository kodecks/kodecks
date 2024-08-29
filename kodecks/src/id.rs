use core::fmt;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

const MAX_RESERVED_ID: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ObjectId(u64);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ObjectId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectIdCounter(u64);

impl Default for ObjectIdCounter {
    fn default() -> Self {
        Self(MAX_RESERVED_ID)
    }
}

impl ObjectIdCounter {
    pub fn allocate(&mut self, base_id: Option<ObjectId>) -> ObjectId {
        match base_id {
            Some(id) if id.0 > 0 && id.0 <= MAX_RESERVED_ID => id,
            _ => {
                self.0 += 1;
                ObjectId(self.0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_tuple, Deserialize_tuple, Hash)]
pub struct TimedObjectId {
    pub id: ObjectId,
    pub timestamp: u64,
}

impl fmt::Display for TimedObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.timestamp)
    }
}

pub trait CardId {
    fn id(&self) -> ObjectId;
    fn timed_id(&self) -> TimedObjectId;
}
