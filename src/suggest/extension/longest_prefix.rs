use std::collections::HashSet;

use priority_container::PrioContainerMax;

use super::{Extension, ExtensionOptions};
use crate::{
    index::SuggestionIndex,
    relevance::{item::EngineItem, RelevanceCalc},
    suggest::query::SuggestionQuery,
};

#[derive(Clone, Copy)]
pub struct LongestPrefixExtension<'a> {
    pub options: ExtensionOptions,
    index: &'a dyn SuggestionIndex,
    pub min_w_len: usize,
    pub max_steps: usize,
}

impl<'a> LongestPrefixExtension<'a> {
    /// Create a new Longest-Prefix Extension
    pub fn new(index: &'a dyn SuggestionIndex, min_w_len: usize, max_steps: usize) -> Self {
        let options = ExtensionOptions::default();
        Self {
            options,
            index,
            min_w_len,
            max_steps,
        }
    }

    fn find_with_longest_prefix(self, inp: &str) -> Vec<EngineItem<'a>> {
        if inp.is_empty() {
            return vec![];
        }

        let mut query = inp;
        let mut steps = 0;

        // TODO: Maybe use prio container here?
        let mut out = vec![];
        let mut already_found = HashSet::with_capacity(self.options.limit);

        loop {
            if steps >= self.max_steps || out.len() >= self.options.limit {
                return out;
            }

            let res = self.index.predictions(query, self.options.limit);
            if !res.is_empty() {
                if out.is_empty() {
                    already_found.extend(out.iter());
                    out.extend(res);
                } else {
                    for o in res {
                        if already_found.contains(&o) {
                            continue;
                        }
                        already_found.insert(o);
                        out.push(o);
                    }
                }
            }

            query = strip_str_end(query, 1);
            let query_len = query.chars().count();
            if query_len < self.min_w_len || query_len == 0 {
                return out;
            }
            steps += 1;
        }
    }
}

impl<'a> Extension<'a> for LongestPrefixExtension<'a> {
    #[inline]
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;

        let longest_items = self.find_with_longest_prefix(&query.query_str);

        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);

        let ordered = query.order_items(longest_items, rel_calc);

        let mut queue = PrioContainerMax::new(self.options.limit);
        queue.extend(ordered);
        queue.into_iter().map(|i| i.0).collect::<Vec<_>>()
    }

    #[inline]
    fn should_run(&self, already_found: usize, _query: &SuggestionQuery) -> bool {
        self.options.enabled && already_found < self.options.threshold
    }

    #[inline]
    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}

/// Returns a substring of `inp` with `len` amount of tailing characters being removed.
/// This works for non UTF-8 as well. If len > |inp| "" gets returned
#[inline]
pub fn strip_str_end(inp: &str, len: usize) -> &str {
    match inp.char_indices().rev().nth(len - 1).map(|i| i.0) {
        Some(end) => &inp[..end],
        None => "",
    }
}
