pub mod custom;
pub mod kanji_align;
pub mod longest_prefix;
pub mod ngram;
pub mod similar_terms;

use super::query::SuggestionQuery;
use crate::relevance::{item::EngineItem, RelevanceCalc, RelevanceWeights};

/// Trait to allow exteding suggestion tasks with custom functionality
pub trait Extension<'a> {
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>>;
    fn should_run(&self, already_found: usize, query: &SuggestionQuery) -> bool;
    fn get_options(&self) -> &ExtensionOptions;

    #[inline]
    fn relevance(&self, item: &EngineItem, str_rel: u16) -> u16 {
        let weights = self.get_options().weights;
        RelevanceCalc::new(weights).calc(item, str_rel)
    }
}

/// Options related to finding results with longest prefix
#[derive(Clone, Copy)]
pub struct ExtensionOptions {
    pub enabled: bool,
    /// Max items this extension is allowed to return
    pub limit: usize,
    /// Max existing results that can exists in order to get this extension to run
    pub threshold: usize,
    pub weights: RelevanceWeights,
    pub min_query_len: usize,
}

impl Default for ExtensionOptions {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 5,
            limit: 30,
            weights: RelevanceWeights::default(),
            min_query_len: 0,
        }
    }
}
