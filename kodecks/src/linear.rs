use std::cmp::Ordering;

use bincode::{Decode, Encode};
use num::{Bounded, NumCast};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum Linear<T> {
    Value(T),
    Modified {
        value: T,
        mul: f64,
        add: f64,
        assign: Option<T>,
    },
}

impl<T> Linear<T> {
    pub fn new(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T> PartialEq for Linear<T>
where
    T: PartialEq + Ord + NumCast + Bounded + Clone + Copy,
{
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl<T> Eq for Linear<T> where T: Eq + Ord + NumCast + Bounded + Clone + Copy {}

impl<T> Linear<T>
where
    T: NumCast + PartialEq + Ord + Bounded + Clone + Copy,
{
    pub fn value(&self) -> T {
        match self {
            Self::Value(value) => *value,
            Self::Modified {
                value,
                mul,
                add,
                assign,
            } => {
                let value = if let Some(assign) = assign {
                    *assign
                } else {
                    *value
                };
                let float: f64 = NumCast::from(value).unwrap_or(0.0);
                let float = float * mul + add;
                NumCast::from(float).unwrap_or(if float <= 0.0 {
                    T::min_value()
                } else {
                    T::max_value()
                })
            }
        }
    }

    fn modified(&self) -> Self {
        if let Self::Value(value) = self {
            Self::Modified {
                value: *value,
                mul: 1.0,
                add: 0.0,
                assign: None,
            }
        } else {
            *self
        }
    }

    pub fn is_modified(&self) -> bool {
        self.diff() != Ordering::Equal
    }

    pub fn diff(&self) -> Ordering {
        if let Self::Modified { value, .. } = self {
            self.value().cmp(value)
        } else {
            Ordering::Equal
        }
    }

    pub fn mul<N>(&mut self, mul: N)
    where
        N: NumCast,
    {
        *self = self.modified();
        if let Self::Modified { mul: m, .. } = self {
            if let Some(mul) = <f64 as NumCast>::from(mul) {
                *m *= mul;
            }
        }
    }

    pub fn add<N>(&mut self, add: N)
    where
        N: NumCast,
    {
        *self = self.modified();
        if let Self::Modified { add: a, .. } = self {
            if let Some(add) = <f64 as NumCast>::from(add) {
                *a += add;
            }
        }
    }

    pub fn assign(&mut self, value: T) {
        *self = self.modified();
        if let Self::Modified { assign, .. } = self {
            *assign = Some(value);
        }
    }
}

impl<T> Default for Linear<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Linear<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Serialize for Linear<T>
where
    T: NumCast + Bounded + PartialEq + Clone + Copy + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Value(value) => SerializedLinear::Value(*value),
            Self::Modified {
                value,
                mul,
                add,
                assign,
            } => SerializedLinear::Modified(*value, *mul, *add, *assign),
        }
        .serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Linear<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Linear<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = SerializedLinear::deserialize(deserializer)?;
        match value {
            SerializedLinear::Value(value) => Ok(Linear::Value(value)),
            SerializedLinear::Modified(value, mul, add, assign) => Ok(Linear::Modified {
                value,
                mul,
                add,
                assign,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode)]
#[serde(untagged)]
enum SerializedLinear<T> {
    Value(T),
    Modified(T, f64, f64, Option<T>),
}
