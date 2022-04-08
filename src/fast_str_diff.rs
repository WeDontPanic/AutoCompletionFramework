use crate::sort_vec::SortVec;

use rayon::prelude::*;

/// Orders a set of values by a custom order function. This caches the result of the
/// order function in oder to keep it as fast as possible
#[derive(Clone)]
pub struct FastStringDist<'a, T> {
    items: Vec<T>,
    comp: &'a str,
    use_parallel: bool,
}

impl<'a, T: Sync + Send> FastStringDist<'a, T> {
    #[inline]
    pub fn new(items: Vec<T>, comp: &'a str) -> Self {
        let parallel = items.len() >= 300 || (comp.len() > 5 && items.len() >= 50);
        Self::new_with_parallel(items, comp, parallel)
    }

    #[inline]
    pub fn new_with_parallel(items: Vec<T>, comp: &'a str, parallel: bool) -> Self {
        Self {
            items,
            comp,
            use_parallel: parallel,
        }
    }

    /// Calls `dist` for each item in `items`. Automatically decides whether to use
    /// multiple threads or not
    pub fn assign_mut<F>(mut self, dist: F) -> Vec<T>
    where
        F: Fn(&mut T, &str) + Send + Sync,
    {
        if self.use_parallel {
            self.items.par_iter_mut().for_each(|i| dist(i, self.comp));
        } else {
            self.items.iter_mut().for_each(|i| dist(i, self.comp));
        }
        self.items
    }

    pub fn ordered<F>(self, dist: F) -> SortVec<T, u16>
    where
        F: Fn(&T, &str) -> u16 + Send + Sync,
    {
        let tmp = if self.use_parallel {
            self.para_order(dist)
        } else {
            self.order(dist)
        };

        SortVec::from_tuple_vec(tmp)
    }

    #[inline(always)]
    fn order<F>(self, dist: F) -> Vec<(T, u16)>
    where
        F: Fn(&T, &str) -> u16 + Send + Sync,
    {
        self.items
            .into_iter()
            .map(|val| {
                let r = dist(&val, self.comp);
                (val, r)
            })
            .collect::<Vec<_>>()
    }

    #[inline(always)]
    fn para_order<F>(self, dist: F) -> Vec<(T, u16)>
    where
        F: Fn(&T, &str) -> u16 + Send + Sync,
    {
        self.items
            .into_par_iter()
            .map(|val| {
                let r = dist(&val, self.comp);
                (val, r)
            })
            .collect::<Vec<_>>()
    }
}
