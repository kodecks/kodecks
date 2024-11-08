use crate::{
    ability::{Ability, AbilityList},
    linear::Linear,
};
use num::{traits::CheckedNeg, Bounded, CheckedDiv, NumCast};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier<T> {
    Assign(T),
    Add(T),
    Sub(T),
    Mul(T),
    Div(T),
}

impl<'de, T> Deserialize<'de> for Modifier<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (op, value): (String, T) = Deserialize::deserialize(deserializer)?;
        match op.as_str() {
            "=" => Ok(Modifier::Assign(value)),
            "+" => Ok(Modifier::Add(value)),
            "-" => Ok(Modifier::Sub(value)),
            "*" => Ok(Modifier::Mul(value)),
            "/" => Ok(Modifier::Div(value)),
            _ => Err(anyhow::anyhow!("Invalid operator: {}", op)).map_err(serde::de::Error::custom),
        }
    }
}

pub trait Modifiable {
    type Value: DeserializeOwned;

    fn modify(&mut self, modifier: Modifier<Self::Value>);
}

impl<T> Modifiable for Linear<T>
where
    T: DeserializeOwned
        + NumCast
        + CheckedNeg
        + CheckedDiv
        + PartialEq
        + Ord
        + Bounded
        + Clone
        + Copy,
{
    type Value = T;

    fn modify(&mut self, modifier: Modifier<T>) {
        match modifier {
            Modifier::Assign(value) => *self = Linear::Value(value),
            Modifier::Add(value) => self.add(value),
            Modifier::Sub(value) => self.sub(value),
            Modifier::Mul(value) => self.mul(value),
            Modifier::Div(value) => self.div(value),
        }
    }
}

impl<T> Modifiable for AbilityList<T>
where
    T: DeserializeOwned + PartialEq + Ability,
{
    type Value = T;
    fn modify(&mut self, modifier: Modifier<Self::Value>) {
        match modifier {
            Modifier::Add(value) => {
                self.add(value);
            }
            Modifier::Sub(value) => {
                self.remove(value);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modify_linear() {
        let mut linear = Linear::new(1);
        linear.modify(Modifier::Assign(4));
        assert_eq!(linear.value(), 4);
        linear.modify(Modifier::Add(5));
        assert_eq!(linear.value(), 9);
        linear.modify(Modifier::Sub(4));
        assert_eq!(linear.value(), 5);
        linear.modify(Modifier::Mul(2));
        assert_eq!(linear.value(), 9);
        linear.modify(Modifier::Div(2));
        assert_eq!(linear.value(), 5);
    }
}
