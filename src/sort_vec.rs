use std::cmp::Ordering;

use order_struct::OrderBy;
use priority_container::PrioContainer;

/// Vec that allows sorting by using a value rather than the (Partial)Ord trait
pub struct SortVec<T, O> {
    inner: Vec<(T, O)>,
}

impl<T, O> SortVec<T, O> {
    /// Creates a new empty SortVec
    #[inline]
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    /// Creates a new sorted vector with given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new SortVec from existing items
    pub fn from_tuple_vec(inner: Vec<(T, O)>) -> Self {
        Self { inner }
    }

    /// Creates a new SortVec from existing items
    pub fn from_vec(inner: Vec<T>, order: Vec<O>) -> Self {
        let inner = inner.into_iter().zip(order.into_iter()).collect();
        Self { inner }
    }

    /// Adds a new value with an assigned `order`
    #[inline]
    pub fn push(&mut self, item: T, ord: O) {
        self.inner.push((item, ord));
    }

    /// Returns an item within the Vec
    #[inline]
    pub fn get(&self, pos: usize) -> Option<&(T, O)> {
        self.inner.get(pos)
    }

    /// Retuns the amount of items in the Vec
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no items in the vec
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over the items mutable
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (T, O)> {
        self.inner.iter_mut()
    }

    /// Returns the inner vec
    #[inline]
    pub fn into_inner(self) -> Vec<(T, O)> {
        self.inner
    }

    /// Iterates over the items borrowed
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &(T, O)> {
        self.inner.iter()
    }
}

/*
impl<T: PartialEq, O> SortVec<T, O> {
    /// Merge this with `other` skipping duplicates
    fn merge(&mut self, other: SortVec<T, O>) {
        // TODO: make this more efficient?
        for i in other.into_inner() {
            let has = self.inner.iter().any(|a| a.0 == i.0);
            if has {
                continue;
            }
            self.push(i.0, i.1);
        }
    }
}
*/

impl<T, O: Ord> SortVec<T, O> {
    /// Returns an iterator over the sorted items
    #[inline]
    pub fn sort(self) -> impl Iterator<Item = T> + DoubleEndedIterator {
        self.sort_ll().into_iter().map(|i| i.0)
    }

    /// Returns an iterator over the sorted items
    #[inline]
    pub fn sort_ll(self) -> Vec<(T, O)> {
        self.sort_by(|a, b| a.1.cmp(&b.1))
    }

    /// Sorts the Vec with a custom order function
    #[inline]
    pub fn sort_by<F>(mut self, ord: F) -> Vec<(T, O)>
    where
        F: Fn(&(T, O), &(T, O)) -> Ordering,
    {
        self.inner.sort_by(ord);
        self.into_inner()
    }

    /// Returns the biggest `n` items borrowed
    #[inline]
    pub fn get_biggest(&self, n: usize) -> Vec<&(T, O)> {
        self.get_biggest_by(n, |a, b| a.1.cmp(&b.1))
    }

    /// Returns the biggest `n` items borrowed using a custom compare function
    #[inline]
    pub fn get_biggest_by<F>(&self, n: usize, order: F) -> Vec<&(T, O)>
    where
        F: Fn(&(T, O), &(T, O)) -> Ordering,
    {
        let mut prio_cont = PrioContainer::new(n);
        for i in self.iter() {
            prio_cont.insert(OrderBy::new(i, |a, b| order(a, b)));
        }
        prio_cont.into_iter().map(|i| i.into_inner()).collect()
    }

    /// Returns the biggest `n` items owned
    #[inline]
    pub fn into_biggest(self, n: usize) -> Vec<(T, O)> {
        self.into_biggest_by(n, |a, b| a.1.cmp(&b.1))
    }

    /// Returns the biggest `n` items owned using a custom compare function
    #[inline]
    pub fn into_biggest_by<F>(self, n: usize, order: F) -> Vec<(T, O)>
    where
        F: Fn(&(T, O), &(T, O)) -> Ordering,
    {
        let mut prio_cont = PrioContainer::new(n);
        for i in self.inner {
            prio_cont.insert(OrderBy::new(i, |a, b| order(a, b)));
        }
        prio_cont.into_iter().map(|i| i.into_inner()).collect()
    }
}

impl<T, O: PartialOrd> Extend<(T, O)> for SortVec<T, O> {
    #[inline]
    fn extend<I: IntoIterator<Item = (T, O)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}

impl<T, O: PartialOrd> FromIterator<(T, O)> for SortVec<T, O> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (T, O)>>(iter: I) -> Self {
        let inner: Vec<_> = iter.into_iter().collect();
        Self { inner }
    }
}
