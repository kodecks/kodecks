use crate::score::Score;
use serde::{Deserialize, Deserializer, Serialize};
use std::ops::{Add, Sub};

mod anonymous;
mod keyword;
mod player;

pub use anonymous::*;
pub use keyword::*;
pub use player::*;

pub trait Ability:
    Copy
    + Clone
    + Ord
    + Add<Output = (Option<Self>, Option<Self>)>
    + Sub<Output = Option<Self>>
    + Score
    + PartialEq
    + Eq
    + Sized
{
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbilityList<T> {
    List(Vec<T>),
    Modified { list: Vec<T>, removed: Vec<T> },
}

impl<T> Default for AbilityList<T>
where
    T: Ability,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Score for AbilityList<T>
where
    T: Ability,
{
    fn score(&self) -> i32 {
        self.iter().map(Score::score).sum()
    }
}

impl<T> FromIterator<T> for AbilityList<T>
where
    T: Ability,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = iter.into_iter().collect::<Vec<_>>();
        list.sort();
        list.dedup();
        Self::List(list)
    }
}

impl<T> AbilityList<T>
where
    T: Ability,
{
    pub fn new() -> Self {
        Self::List(Vec::new())
    }

    pub fn len(&self) -> usize {
        match self {
            Self::List(list) => list.len(),
            Self::Modified { list, .. } => list.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        match self {
            Self::List(list) => list.iter(),
            Self::Modified { list, .. } => list.iter(),
        }
    }

    pub fn contains(&self, ability: &T) -> bool {
        match self {
            Self::List(list) => list.contains(ability),
            Self::Modified { list, .. } => list.contains(ability),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::List(list) => list.is_empty(),
            Self::Modified { list, .. } => list.is_empty(),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::List(list) => list.clear(),
            Self::Modified { list, removed } => {
                list.clear();
                removed.clear();
            }
        }
    }

    fn make_modified(&mut self) {
        if let Self::List(list) = self {
            *self = Self::Modified {
                list: std::mem::take(list),
                removed: Vec::new(),
            };
        }
    }

    pub fn add(&mut self, ability: T) {
        self.make_modified();
        if let Self::Modified { list, removed } = self {
            let mut ability = Some(ability);
            list.retain_mut(|a| {
                if let Some(added) = ability {
                    let (updated, output) = *a + added;
                    ability = output;
                    if let Some(updated) = updated {
                        *a = updated;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            });
            if let Some(added) = ability {
                list.push(added);
                list.sort();
                list.dedup();
            }
            for removed in removed.iter() {
                list.retain_mut(|a| {
                    if let Some(result) = *a - *removed {
                        *a = result;
                        true
                    } else {
                        false
                    }
                });
            }
        }
    }

    pub fn remove(&mut self, ability: T) {
        self.make_modified();
        if let Self::Modified { list, removed } = self {
            list.retain_mut(|a| {
                if let Some(result) = *a - ability {
                    *a = result;
                    true
                } else {
                    false
                }
            });
            removed.push(ability);
            removed.sort();
            removed.dedup();
        }
    }
}

impl<T> AsRef<[T]> for AbilityList<T> {
    fn as_ref(&self) -> &[T] {
        match self {
            Self::List(list) => list.as_ref(),
            Self::Modified { list, .. } => list.as_ref(),
        }
    }
}

impl<T> Serialize for AbilityList<T>
where
    T: Serialize + Ability,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for AbilityList<T>
where
    T: Deserialize<'de> + Ability,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut abilities = Vec::<T>::deserialize(deserializer)?;
        abilities.sort();
        abilities.dedup();
        Ok(Self::List(abilities))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyword::KeywordAbility;
    use player::PlayerAbility;

    #[test]
    fn test_keyword_ability_list() {
        let mut list = AbilityList::<KeywordAbility>::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        list.add(KeywordAbility::Toxic);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&KeywordAbility::Toxic));

        list.add(KeywordAbility::Toxic);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&KeywordAbility::Toxic));

        list.add(KeywordAbility::Toxic);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&KeywordAbility::Toxic));

        list.remove(KeywordAbility::Toxic);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&KeywordAbility::Toxic));

        list.remove(KeywordAbility::Toxic);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&KeywordAbility::Toxic));

        list.add(KeywordAbility::Toxic);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&KeywordAbility::Toxic));

        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_player_ability_list() {
        let mut list = AbilityList::<PlayerAbility>::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        list.add(PlayerAbility::Propagate(1));
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&PlayerAbility::Propagate(1)));

        list.add(PlayerAbility::Propagate(2));
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&PlayerAbility::Propagate(3)));

        list.add(PlayerAbility::Propagate(3));
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&PlayerAbility::Propagate(6)));

        list.add(PlayerAbility::Propagate(-10));
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&PlayerAbility::Propagate(-4)));

        list.add(PlayerAbility::Propagate(4));
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&PlayerAbility::Propagate(0)));

        list.add(PlayerAbility::Propagate(4));
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.contains(&PlayerAbility::Propagate(4)));

        list.remove(PlayerAbility::Propagate(0));
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&PlayerAbility::Propagate(0)));

        list.add(PlayerAbility::Propagate(4));
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(!list.contains(&PlayerAbility::Propagate(0)));
    }
}
