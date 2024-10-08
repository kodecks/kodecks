use crate::ability::KeywordAbility;
use crate::error::ActionError;
use bincode::{Decode, Encode};
use fluent_bundle::{FluentArgs, FluentValue};
use serde::{de, ser, Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Encode, Decode)]
#[non_exhaustive]
#[serde(untagged, rename_all = "snake_case")]
pub enum Value {
    Ability(KeywordAbility),
    Integer(i32),
}

impl Value {
    pub fn get<T>(&self) -> Option<T>
    where
        T: TryFrom<Value>,
    {
        (*self).try_into().ok()
    }

    pub fn fluent_value(&self) -> FluentValue {
        match self {
            Value::Ability(ability) => FluentValue::String(Cow::Borrowed(ability.into())),
            Value::Integer(int) => FluentValue::Number((*int).into()),
        }
    }
}

impl TryFrom<Value> for KeywordAbility {
    type Error = ActionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Ability(ability) => Ok(ability),
            _ => Err(ActionError::InvalidValueType),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = ActionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(int) => Ok(int),
            _ => Err(ActionError::InvalidValueType),
        }
    }
}

impl From<KeywordAbility> for Value {
    fn from(ability: KeywordAbility) -> Self {
        Value::Ability(ability)
    }
}

impl From<i32> for Value {
    fn from(int: i32) -> Self {
        Value::Integer(int)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Encode, Decode)]
pub struct VariableList(Vec<(String, Value)>);

impl VariableList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn set<K, T>(mut self, key: K, value: T) -> Self
    where
        K: ToString,
        T: Into<Value>,
    {
        self.0.push((key.to_string(), value.into()));
        self
    }

    pub fn insert<K, T>(&mut self, key: K, value: T)
    where
        K: ToString,
        T: Into<Value>,
    {
        self.0.push((key.to_string(), value.into()));
    }

    pub fn get<T>(&self, key: &str) -> Result<T, ActionError>
    where
        T: TryFrom<Value, Error = ActionError>,
    {
        let (_, value) = self
            .0
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .ok_or_else(|| ActionError::KeyNotFound {
                key: key.to_string(),
            })?;
        (*value).try_into()
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        if let Some(index) = self.0.iter().position(|(k, _)| k.as_str() == key) {
            return Some(self.0.remove(index).1);
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut Value)> {
        self.0.iter_mut().map(|(k, v)| (k.as_str(), v))
    }

    pub fn fluent_args(&self) -> FluentArgs {
        let mut args = FluentArgs::new();
        for (key, value) in self.0.iter() {
            args.set(key.as_str(), value.fluent_value());
        }
        args
    }
}

impl IntoIterator for VariableList {
    type Item = (String, Value);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter().map(|(k, v)| (k.to_string(), v)))
    }
}

impl ser::Serialize for VariableList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0.iter() {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl<'de> de::Deserialize<'de> for VariableList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use de::MapAccess;
        struct VariableListVisitor;

        impl<'de> de::Visitor<'de> for VariableListVisitor {
            type Value = VariableList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of variables")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some((key, value)) = map.next_entry()? {
                    values.push((key, value));
                }
                Ok(VariableList(values))
            }
        }

        deserializer.deserialize_map(VariableListVisitor)
    }
}
