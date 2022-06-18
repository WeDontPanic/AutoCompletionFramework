/// Generic index
pub mod basic;
/// Index for Japanese terms
pub mod japanese;
/// Raw N-gram based index
pub mod ngram;
pub mod output;
pub mod str_item;

pub use output::Output;

use crate::relevance::item::EngineItem;

/// Item in an index. Must be convertable to Output
pub trait IndexItem: ToOutput + Send + Sync {
    fn frequency(&self) -> f64;
    fn word_id(&self) -> u32;
    fn str_relevance(&self, s: &str) -> u16;
    fn into_engine_item(&self) -> EngineItem;
    fn terms(&self) -> Vec<&String>;
}

/// Convert anything to `Output`
pub trait ToOutput {
    fn to_output(&self) -> Output;
}

pub trait SuggestionIndex {
    fn predictions(&self, inp: &str, limit: usize) -> Vec<EngineItem>;
    fn exact(&self, inp: &str) -> Vec<EngineItem>;
    fn get_word(&self, id: u32) -> Option<EngineItem>;

    #[inline]
    fn similar_terms(&self, _inp: &str, _limit: usize, _max_dist: u32) -> Vec<EngineItem> {
        vec![]
    }

    fn len(&self) -> usize {
        0
    }
}

pub trait KanjiReadingAlign {
    fn align_reading(&self, query: &str) -> Vec<EngineItem>;
}

pub trait NGIndexable {
    fn similar(
        &self,
        query: &str,
        limit: usize,
        q_weight: f32,
        term_limit: usize,
    ) -> Vec<EngineItem>;
}
