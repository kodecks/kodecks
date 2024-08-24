use crate::{effect::StackEffectHandler, id::ObjectId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct StackItem {
    pub source: ObjectId,
    pub id: String,
    pub handler: Arc<Box<StackEffectHandler>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct LocalStackItem {
    pub source: ObjectId,
    pub id: String,
}

impl From<StackItem> for LocalStackItem {
    fn from(item: StackItem) -> Self {
        Self {
            source: item.source,
            id: item.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Extend<T> for Stack<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let items = iter.into_iter().collect::<Vec<_>>();
        Self { items }
    }
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn as_slice(&self) -> &[T] {
        self.items.as_slice()
    }
}
