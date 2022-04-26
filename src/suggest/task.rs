use std::collections::HashSet;

use super::query::SuggestionQuery;
use priority_container::PrioContainerMax;

use crate::{
    index::{IndexItem, Output},
    relevance::item::EngineItem,
};

/// An autocompletion task to run multiple suggestion queries
pub struct SuggestionTask<'index, 'a, 'ext> {
    queries: Vec<SuggestionQuery<'index, 'ext>>,
    custom_entries: Vec<EngineItem<'a>>,
    limit: usize,
    debug: bool,
    filter: Option<Box<dyn Fn(&dyn IndexItem) -> bool + 'ext>>,
}

impl<'index, 'a, 'ext> SuggestionTask<'index, 'a, 'ext> {
    /// Create a new SuggestionTask with the given output item limit
    pub fn new(limit: usize) -> Self {
        Self {
            queries: vec![],
            limit,
            custom_entries: vec![],
            debug: false,
            filter: None,
        }
    }

    /// Sets a filter for output items
    pub fn set_filter<F: Fn(&dyn IndexItem) -> bool + 'ext>(&mut self, filter: F) {
        self.filter = Some(Box::new(filter))
    }

    // Adds a query to the Task
    pub fn add_query(&mut self, query: SuggestionQuery<'index, 'ext>) {
        self.queries.push(query);
    }

    /// Adds custom items to the result
    pub fn add_custom_entries(&mut self, entries: Vec<EngineItem<'a>>) {
        self.custom_entries.extend(entries);
    }

    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// Performs the suggestion search
    pub fn search(&self) -> Vec<Output> {
        let mut out = PrioContainerMax::new(self.limit);
        let mut added_items = HashSet::with_capacity(self.limit);
        let mut added = 0;

        for query in &self.queries {
            if added >= query.threshold && query.threshold > 0 {
                continue;
            }

            let query_res = query.search(self.limit);

            if self.debug {
                println!("query found {} items", query_res.len());
            }

            added += query_res.len();
            for i in query_res.into_iter().filter(|i| self.item_allowed(i)) {
                out.insert(i);
                added_items.insert(i);
            }
        }

        for i in &self.custom_entries {
            if !self.item_allowed(i) || added_items.contains(i) {
                continue;
            }
            added_items.insert(*i);
            out.insert(*i);
        }

        let mut out = out
            .into_iter()
            .inspect(|i| {
                if self.debug {
                    println!("{i:?} (w-freq: {})", i.0.inner().frequency());
                }
            })
            .map(|i| i.0.inner().to_output())
            .collect::<Vec<_>>();
        out.reverse();
        out
    }

    /// Returns `true` if the item `i` should not get filtered out
    #[inline]
    fn item_allowed(&self, i: &EngineItem) -> bool {
        self.filter.as_ref().map(|f| f(*i.inner())).unwrap_or(true)
    }

    /// Returns the amount of queries
    #[inline]
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Returns `true` if there is no query added to the SuggestionTask
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
