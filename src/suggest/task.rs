use super::query::SuggestionQuery;
use order_struct::order_nh::OrderVal;
use priority_container::unique::UniquePrioContainerMax;

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
    rel_mod: Option<Box<dyn Fn(&EngineItem, u16) -> u16 + 'ext>>,
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
            rel_mod: None,
        }
    }

    /// Sets a filter for output items
    pub fn set_rel_mod<F: Fn(&EngineItem, u16) -> u16 + 'ext>(&mut self, rel_mod: F) {
        self.rel_mod = Some(Box::new(rel_mod))
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
        let mut out = UniquePrioContainerMax::new(self.limit);
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
                let i = self.apply_rel_mod(i);
                out.insert(OrderVal::new(i.to_output(), i.get_relevance()));
            }
        }

        let cust_add = self
            .custom_entries
            .iter()
            .filter(|i| self.item_allowed(i))
            .map(|i| self.apply_rel_mod(*i))
            .map(|i| OrderVal::new(i.to_output(), i.get_relevance()));
        out.extend(cust_add);

        let mut out = out
            .into_iter()
            .map(|i| i.0.into_inner())
            .collect::<Vec<_>>();
        out.reverse();
        out
    }

    #[inline]
    fn apply_rel_mod<'r>(&self, mut item: EngineItem<'r>) -> EngineItem<'r> {
        if let Some(ref rel_mod) = self.rel_mod {
            let rel = rel_mod(&item, item.get_relevance());
            item.set_relevance(rel);
        }
        item
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
