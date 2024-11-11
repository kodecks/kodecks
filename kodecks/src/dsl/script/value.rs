use super::{
    error::Error,
    exp::{ExpEnv, Function},
};
use crate::id::TimedObjectId;
use serde_json::Number;
use std::{
    collections::BTreeMap,
    fmt,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};
use tinystr::TinyAsciiStr;

#[derive(Debug, Clone)]
pub enum Value {
    Constant(Constant),
    Array(Vec<Self>),
    Object(BTreeMap<TinyAsciiStr<32>, Self>),
    Function(Box<Function>),
    Custom(CustomType),
}

impl Default for Value {
    fn default() -> Self {
        Value::Constant(Constant::Null)
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Constant(Constant::Null),
            serde_json::Value::Bool(b) => Value::Constant(Constant::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(n) = n.as_u64() {
                    Value::Constant(Constant::U64(n))
                } else if let Some(n) = n.as_i64() {
                    Value::Constant(Constant::I64(n))
                } else {
                    Value::Constant(Constant::F64(n.as_f64().unwrap()))
                }
            }
            serde_json::Value::String(s) => Value::Constant(Constant::String(
                TinyAsciiStr::from_bytes_lossy(s.as_bytes()),
            )),
            serde_json::Value::Array(a) => Value::Array(a.into_iter().map(Value::from).collect()),
            serde_json::Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| (TinyAsciiStr::from_bytes_lossy(k.as_bytes()), Value::from(v)))
                    .collect(),
            ),
        }
    }
}

impl TryFrom<Value> for serde_json::Value {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Constant(Constant::Null) => Ok(serde_json::Value::Null),
            Value::Constant(Constant::Bool(b)) => Ok(serde_json::Value::Bool(b)),
            Value::Constant(Constant::U64(n)) => Ok(serde_json::Value::Number(n.into())),
            Value::Constant(Constant::I64(n)) => Ok(serde_json::Value::Number(n.into())),
            Value::Constant(Constant::F64(n)) => Ok(serde_json::Value::Number(
                Number::from_f64(n).ok_or(Error::InvalidConversion)?,
            )),
            Value::Constant(Constant::String(s)) => Ok(serde_json::Value::String(s.to_string())),
            Value::Array(a) => Ok(serde_json::Value::Array(
                a.into_iter()
                    .map(serde_json::Value::try_from)
                    .collect::<Result<_, _>>()?,
            )),
            Value::Object(o) => Ok(serde_json::Value::Object(
                o.into_iter()
                    .map(|(k, v)| Ok((k.to_string(), serde_json::Value::try_from(v)?)))
                    .collect::<Result<_, _>>()?,
            )),
            Value::Custom(CustomType::Card(card)) => {
                Ok(serde_json::Value::Number(Into::<u64>::into(card).into()))
            }
            Value::Custom(CustomType::Player(player)) => {
                Ok(serde_json::Value::Number(player.into()))
            }
            _ => Err(Error::InvalidConversion),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Constant {
    #[default]
    Null,
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    String(TinyAsciiStr<32>),
}

impl Constant {
    fn kind(&self) -> ValueKind {
        match self {
            Constant::Null => ValueKind::Null,
            Constant::Bool(false) => ValueKind::False,
            Constant::Bool(true) => ValueKind::True,
            Constant::U64(_) | Constant::I64(_) | Constant::F64(_) => ValueKind::Number,
            Constant::String(_) => ValueKind::String,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Constant::U64(n) => Some(*n),
            Constant::I64(n) => (*n).try_into().ok(),
            Constant::F64(n) => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Constant::U64(n) => (*n).try_into().ok(),
            Constant::I64(n) => Some(*n),
            Constant::F64(n) => Some(*n as i64),
            _ => None,
        }
    }
}

impl Add for Constant {
    type Output = Result<Self, Error>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::String(a), Constant::String(b)) => {
                let mut a = a.to_string();
                a.push_str(&b);
                Ok(Constant::String(TinyAsciiStr::from_bytes_lossy(
                    a.as_bytes(),
                )))
            }
            (Constant::U64(a), Constant::U64(b)) => Ok(Constant::U64(a + b)),
            (Constant::U64(a), Constant::I64(b)) => Ok(Constant::I64(a as i64 + b)),
            (Constant::U64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 + b)),
            (Constant::I64(a), Constant::U64(b)) => Ok(Constant::I64(a + b as i64)),
            (Constant::I64(a), Constant::I64(b)) => Ok(Constant::I64(a + b)),
            (Constant::I64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 + b)),
            (Constant::F64(a), Constant::F64(b)) => Ok(Constant::F64(a + b)),
            (Constant::F64(a), Constant::U64(b)) => Ok(Constant::F64(a + b as f64)),
            (Constant::F64(a), Constant::I64(b)) => Ok(Constant::F64(a + b as f64)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Sub for Constant {
    type Output = Result<Self, Error>;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::U64(a), Constant::U64(b)) => Ok(Constant::I64(a as i64 - b as i64)),
            (Constant::U64(a), Constant::I64(b)) => Ok(Constant::I64(a as i64 - b)),
            (Constant::U64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 - b)),
            (Constant::I64(a), Constant::U64(b)) => Ok(Constant::I64(a - b as i64)),
            (Constant::I64(a), Constant::I64(b)) => Ok(Constant::I64(a - b)),
            (Constant::I64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 - b)),
            (Constant::F64(a), Constant::F64(b)) => Ok(Constant::F64(a - b)),
            (Constant::F64(a), Constant::U64(b)) => Ok(Constant::F64(a - b as f64)),
            (Constant::F64(a), Constant::I64(b)) => Ok(Constant::F64(a - b as f64)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Mul for Constant {
    type Output = Result<Self, Error>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Constant::U64(a), Constant::U64(b)) => Ok(Constant::U64(a * b)),
            (Constant::U64(a), Constant::I64(b)) => Ok(Constant::I64(a as i64 * b)),
            (Constant::U64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 * b)),
            (Constant::I64(a), Constant::U64(b)) => Ok(Constant::I64(a * b as i64)),
            (Constant::I64(a), Constant::I64(b)) => Ok(Constant::I64(a * b)),
            (Constant::I64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 * b)),
            (Constant::F64(a), Constant::F64(b)) => Ok(Constant::F64(a * b)),
            (Constant::F64(a), Constant::U64(b)) => Ok(Constant::F64(a * b as f64)),
            (Constant::F64(a), Constant::I64(b)) => Ok(Constant::F64(a * b as f64)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Div for Constant {
    type Output = Result<Self, Error>;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (_, Constant::U64(0)) => Err(Error::DivisionByZero),
            (_, Constant::I64(0)) => Err(Error::DivisionByZero),
            (_, Constant::F64(0.0)) => Err(Error::DivisionByZero),
            (Constant::U64(a), Constant::U64(b)) => Ok(Constant::F64(a as f64 / b as f64)),
            (Constant::U64(a), Constant::I64(b)) => Ok(Constant::F64(a as f64 / b as f64)),
            (Constant::U64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 / b)),
            (Constant::I64(a), Constant::U64(b)) => Ok(Constant::F64(a as f64 / b as f64)),
            (Constant::I64(a), Constant::I64(b)) => Ok(Constant::F64(a as f64 / b as f64)),
            (Constant::I64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 / b)),
            (Constant::F64(a), Constant::F64(b)) => Ok(Constant::F64(a / b)),
            (Constant::F64(a), Constant::U64(b)) => Ok(Constant::F64(a / b as f64)),
            (Constant::F64(a), Constant::I64(b)) => Ok(Constant::F64(a / b as f64)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Rem for Constant {
    type Output = Result<Self, Error>;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (_, Constant::U64(0)) => Err(Error::DivisionByZero),
            (_, Constant::I64(0)) => Err(Error::DivisionByZero),
            (_, Constant::F64(0.0)) => Err(Error::DivisionByZero),
            (Constant::U64(a), Constant::U64(b)) => Ok(Constant::U64(a % b)),
            (Constant::U64(a), Constant::I64(b)) => Ok(Constant::I64(a as i64 % b)),
            (Constant::U64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 % b)),
            (Constant::I64(a), Constant::U64(b)) => Ok(Constant::I64(a % b as i64)),
            (Constant::I64(a), Constant::I64(b)) => Ok(Constant::I64(a % b)),
            (Constant::I64(a), Constant::F64(b)) => Ok(Constant::F64(a as f64 % b)),
            (Constant::F64(a), Constant::F64(b)) => Ok(Constant::F64(a % b)),
            (Constant::F64(a), Constant::U64(b)) => Ok(Constant::F64(a % b as f64)),
            (Constant::F64(a), Constant::I64(b)) => Ok(Constant::F64(a % b as f64)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Neg for Constant {
    type Output = Result<Self, Error>;

    fn neg(self) -> Self::Output {
        match self {
            Constant::U64(n) => {
                let i: Option<i64> = n.try_into().ok();
                if let Some(i) = i {
                    Ok(Constant::I64(-i))
                } else {
                    Ok(Constant::F64(-(n as f64)))
                }
            }
            Constant::I64(n) => Ok(Constant::I64(-n)),
            Constant::F64(n) => Ok(Constant::F64(-n)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Constant::Null, Constant::Null) => Some(std::cmp::Ordering::Equal),
            (Constant::Bool(a), Constant::Bool(b)) => a.partial_cmp(b),
            (Constant::U64(a), Constant::U64(b)) => a.partial_cmp(b),
            (Constant::U64(a), Constant::I64(b)) => (*a as f64).partial_cmp(&(*b as f64)),
            (Constant::U64(a), Constant::F64(b)) => (*a as f64).partial_cmp(b),
            (Constant::I64(a), Constant::U64(b)) => (*a as f64).partial_cmp(&(*b as f64)),
            (Constant::I64(a), Constant::I64(b)) => a.partial_cmp(b),
            (Constant::I64(a), Constant::F64(b)) => (*a as f64).partial_cmp(b),
            (Constant::F64(a), Constant::F64(b)) => a.partial_cmp(b),
            (Constant::F64(a), Constant::U64(b)) => a.partial_cmp(&(*b as f64)),
            (Constant::F64(a), Constant::I64(b)) => a.partial_cmp(&(*b as f64)),
            (Constant::String(a), Constant::String(b)) => a.partial_cmp(b),
            (a, b) => a.kind().partial_cmp(&b.kind()),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Null, Constant::Null) => true,
            (Constant::Bool(a), Constant::Bool(b)) => a == b,
            (Constant::U64(a), Constant::U64(b)) => a == b,
            (Constant::U64(a), Constant::I64(b)) => *a == *b as u64,
            (Constant::U64(a), Constant::F64(b)) => *a as f64 == *b,
            (Constant::I64(a), Constant::U64(b)) => *a as u64 == *b,
            (Constant::I64(a), Constant::I64(b)) => a == b,
            (Constant::I64(a), Constant::F64(b)) => *a as f64 == *b,
            (Constant::F64(a), Constant::F64(b)) => a == b,
            (Constant::F64(a), Constant::U64(b)) => *a == *b as f64,
            (Constant::F64(a), Constant::I64(b)) => *a == *b as f64,
            (Constant::String(a), Constant::String(b)) => a == b,
            _ => false,
        }
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Constant::Bool(value)
    }
}

impl From<u32> for Constant {
    fn from(value: u32) -> Self {
        Constant::U64(value.into())
    }
}

impl From<i32> for Constant {
    fn from(value: i32) -> Self {
        Constant::I64(value.into())
    }
}

impl From<u64> for Constant {
    fn from(value: u64) -> Self {
        Constant::U64(value)
    }
}

impl From<i64> for Constant {
    fn from(value: i64) -> Self {
        Constant::I64(value)
    }
}

impl From<f64> for Constant {
    fn from(value: f64) -> Self {
        Constant::F64(value)
    }
}

impl From<TinyAsciiStr<32>> for Constant {
    fn from(value: TinyAsciiStr<32>) -> Self {
        Constant::String(value)
    }
}

impl From<String> for Constant {
    fn from(value: String) -> Self {
        Constant::String(TinyAsciiStr::from_bytes_lossy(value.as_bytes()))
    }
}

impl From<&str> for Constant {
    fn from(value: &str) -> Self {
        Constant::String(TinyAsciiStr::from_bytes_lossy(value.as_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CustomType {
    Card(TimedObjectId),
    Player(u8),
}

#[derive(PartialEq, PartialOrd)]
enum ValueKind {
    Null,
    False,
    True,
    Number,
    String,
    Array,
    Object,
    Function,
    Custom,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                if a == b {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    a.len().partial_cmp(&b.len())
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                if a == b {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    a.len().partial_cmp(&b.len())
                }
            }
            (Value::Constant(a), Value::Constant(b)) => a.partial_cmp(b),
            (Value::Custom(a), Value::Custom(b)) => a.partial_cmp(b),
            (a, b) => a.kind().partial_cmp(&b.kind()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Constant(a), Value::Constant(b)) => a == b,
            (Value::Custom(a), Value::Custom(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl<T> From<T> for Value
where
    T: Into<Constant>,
{
    fn from(value: T) -> Self {
        Value::Constant(value.into())
    }
}

impl From<TimedObjectId> for Value {
    fn from(value: TimedObjectId) -> Self {
        Value::Custom(CustomType::Card(value))
    }
}

impl Add for Value {
    type Output = Result<Self, Error>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (a, Value::Constant(Constant::Null)) => Ok(a),
            (Value::Constant(Constant::Null), b) => Ok(b),
            (Value::Constant(a), Value::Constant(b)) => Ok(Value::Constant((a + b)?)),
            (Value::Object(mut a), Value::Object(b)) => {
                a.extend(b);
                Ok(Value::Object(a))
            }
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, Error>;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Constant(a), Value::Constant(b)) => Ok(Value::Constant((a - b)?)),
            (Value::Array(mut a), Value::Array(b)) => {
                a.retain(|item| !b.contains(item));
                Ok(Value::Array(a))
            }
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, Error>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Constant(Constant::String(a)), Value::Constant(Constant::U64(b))) => {
                let a = a.to_string().repeat(b as usize);
                Ok(Value::Constant(Constant::String(
                    TinyAsciiStr::from_bytes_lossy(a.as_bytes()),
                )))
            }
            (Value::Constant(a), Value::Constant(b)) => Ok(Value::Constant((a * b)?)),
            (Value::Object(a), Value::Object(b)) => {
                let mut a = Value::Object(a);
                merge_values(&mut a, &Value::Object(b));
                Ok(a)
            }
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Div for Value {
    type Output = Result<Self, Error>;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Constant(Constant::String(a)), Value::Constant(Constant::String(b))) => {
                let a = a.to_string();
                let b = b.to_string();

                Ok(Value::Array(a.split(&b).map(|s| s.into()).collect()))
            }
            (Value::Constant(a), Value::Constant(b)) => Ok(Value::Constant((a / b)?)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Rem for Value {
    type Output = Result<Self, Error>;

    fn rem(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Constant(a), Value::Constant(b)) => Ok(Value::Constant((a % b)?)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Neg for Value {
    type Output = Result<Self, Error>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Constant(a) => Ok(Value::Constant((-a)?)),
            _ => Err(Error::InvalidCalculation),
        }
    }
}

impl Not for Value {
    type Output = bool;

    fn not(self) -> Self::Output {
        match self {
            Value::Constant(Constant::Null) => true,
            Value::Constant(Constant::Bool(b)) => !b,
            _ => false,
        }
    }
}

impl Not for &Value {
    type Output = bool;

    fn not(self) -> Self::Output {
        match self {
            Value::Constant(Constant::Null) => true,
            Value::Constant(Constant::Bool(b)) => !b,
            _ => false,
        }
    }
}

impl Value {
    fn kind(&self) -> ValueKind {
        match self {
            Value::Constant(Constant::Null) => ValueKind::Null,
            Value::Constant(Constant::Bool(false)) => ValueKind::False,
            Value::Constant(Constant::Bool(true)) => ValueKind::True,
            Value::Constant(Constant::U64(_))
            | Value::Constant(Constant::I64(_))
            | Value::Constant(Constant::F64(_)) => ValueKind::Number,
            Value::Constant(Constant::String(_)) => ValueKind::String,
            Value::Array(_) => ValueKind::Array,
            Value::Object(_) => ValueKind::Object,
            Value::Function(_) => ValueKind::Function,
            Value::Custom(_) => ValueKind::Custom,
        }
    }

    pub fn index_range(&self, start: Option<i64>, end: Option<i64>) -> Result<Self, Error> {
        match self {
            Value::Array(array) => {
                let start = start.unwrap_or(0);
                let end = end.unwrap_or(array.len() as i64);
                let start = if start < 0 {
                    array.len() as i64 + start
                } else {
                    start
                } as usize;
                let end = if end < 0 {
                    array.len() as i64 + end
                } else {
                    end
                } as usize;
                Ok(Value::Array(
                    array
                        .iter()
                        .skip(start)
                        .take(end - start)
                        .cloned()
                        .collect(),
                ))
            }
            Value::Constant(Constant::String(s)) => {
                let len = s.chars().count() as i64;
                let start = start.unwrap_or(0);
                let end = end.unwrap_or(len);
                let start = if start < 0 { len + start } else { start } as usize;
                let end = if end < 0 { len + end } else { end } as usize;
                Ok(Value::Constant(
                    s.chars()
                        .skip(start)
                        .take(end - start)
                        .collect::<String>()
                        .into(),
                ))
            }
            _ => Err(Error::InvalidKey),
        }
    }

    pub fn index_num(&self, index: i64) -> Result<Self, Error> {
        match self {
            Value::Array(array) => {
                let index = if index < 0 {
                    array.len() as i64 + index
                } else {
                    index
                } as usize;
                Ok(array
                    .get(index)
                    .cloned()
                    .unwrap_or(Value::Constant(Constant::Null)))
            }
            _ => Err(Error::InvalidKey),
        }
    }

    pub fn index_str<E>(&self, index: &TinyAsciiStr<32>, env: &E) -> Result<Self, Error>
    where
        E: ExpEnv,
    {
        match self {
            Value::Object(object) => Ok(object
                .get(index)
                .cloned()
                .unwrap_or(Value::Constant(Constant::Null))),
            Value::Custom(CustomType::Card(card)) => Ok(env
                .get_card(*card)
                .and_then(|card| match index.as_str() {
                    "name" => Some(card.archetype().name.clone().into()),
                    _ => None,
                })
                .unwrap_or_default()),
            _ => Err(Error::InvalidKey),
        }
    }
}

fn merge_values(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            for (key, b_value) in b {
                match a.get_mut(key) {
                    Some(a_value) => merge_values(a_value, b_value),
                    None => {
                        a.insert(*key, b_value.clone());
                    }
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Array(array) => {
                write!(f, "[")?;
                for (i, value) in array.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
            Value::Object(object) => {
                write!(f, "{{")?;
                for (i, (key, value)) in object.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Constant(Constant::Null) => write!(f, "null"),
            Value::Constant(Constant::Bool(b)) => write!(f, "{}", b),
            Value::Constant(Constant::U64(n)) => write!(f, "{}", n),
            Value::Constant(Constant::I64(n)) => write!(f, "{}", n),
            Value::Constant(Constant::F64(n)) => write!(f, "{}", n),
            Value::Constant(Constant::String(s)) => write!(f, "{}", s),
            Value::Function(function) => write!(f, "func {}", function.name),
            Value::Custom(CustomType::Card(card)) => write!(f, "{}", card),
            Value::Custom(CustomType::Player(player)) => write!(f, "Player {}", player),
        }
    }
}
