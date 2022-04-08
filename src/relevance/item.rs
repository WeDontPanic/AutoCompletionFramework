use std::{fmt::Debug, hash::Hash};

use crate::index::{IndexItem, Output};
use order_struct::OrderVal;

/// Wrapper around IndexItem with ordering
#[derive(PartialOrd, Ord, Clone, Copy)]
pub struct EngineItem<'a> {
    item: OrderVal<&'a dyn IndexItem, u16>,
}

impl<'a> EngineItem<'a> {
    #[inline(always)]
    pub fn new(val: &'a dyn IndexItem, relevance: u16) -> Self {
        Self {
            item: OrderVal::new(val, relevance),
        }
    }

    /// Convert to output
    #[inline]
    pub fn to_output(self) -> Output {
        self.item.inner().to_output()
    }

    #[inline]
    pub fn inner(&self) -> &&dyn IndexItem {
        self.item.inner()
    }

    /// Set the order to a new value
    #[inline]
    pub fn set_relevance(&mut self, new_val: u16) {
        self.item.set_ord(new_val)
    }

    /// Get the order value of the item
    #[inline]
    pub fn get_relevance(&self) -> u16 {
        *self.item.ord()
    }
}

impl<'a> Debug for EngineItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = self.item.inner().to_output();
        write!(f, "{out:?}: ({})", self.item.ord())
    }
}

impl<'a> Hash for EngineItem<'a> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for term in self.item.inner().terms() {
            term.hash(state);
        }
        self.inner().word_id().hash(state);
    }
}

impl<'a> PartialEq for EngineItem<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner().terms() == other.inner().terms()
            && self.inner().word_id() == other.inner().word_id()
    }
}

impl<'a> Eq for EngineItem<'a> {}
