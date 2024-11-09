use crate::error::ActionError;
use bincode::{Decode, Encode};
use core::fmt;
use serde::{de, ser, Deserialize, Serialize};
use std::num::NonZeroU32;

const MAX_RESERVED_ID: u32 = 100;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Encode, Decode,
)]
#[serde(transparent)]
pub struct ObjectId(NonZeroU32);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u32> for ObjectId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(value).map(Self).ok_or(())
    }
}

impl From<ObjectId> for u32 {
    fn from(id: ObjectId) -> Self {
        id.0.get()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectIdCounter(u32);

impl Default for ObjectIdCounter {
    fn default() -> Self {
        Self(MAX_RESERVED_ID)
    }
}

impl ObjectIdCounter {
    pub fn allocate(&mut self, base_id: Option<ObjectId>) -> ObjectId {
        match base_id {
            Some(id) => id,
            _ => {
                self.0 += 1;
                ObjectId(NonZeroU32::new(self.0).unwrap())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct TimedObjectId {
    pub id: ObjectId,
    pub timestamp: u16,
}

impl From<TimedObjectId> for u64 {
    fn from(id: TimedObjectId) -> Self {
        (u64::from(id.timestamp) << 32) | u64::from(id.id.0.get())
    }
}

impl TryFrom<u64> for TimedObjectId {
    type Error = ActionError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let id = value as u32;
        let timestamp = (value >> 32) as u16;
        Ok(TimedObjectId {
            id: ObjectId(NonZeroU32::new(id).ok_or(ActionError::InvalidObjectId)?),
            timestamp,
        })
    }
}

impl ser::Serialize for TimedObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Into::<u64>::into(*self).serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for TimedObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let id = u64::deserialize(deserializer)?;
        id.try_into().map_err(de::Error::custom)
    }
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
