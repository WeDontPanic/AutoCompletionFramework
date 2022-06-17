pub mod basic;
pub mod japanese;
pub mod ngram;
pub mod ngram_ext;
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

    /*
    #[inline]
    fn str_relevance(&self, id: u32, query: &str) -> u16 {
        self.get_word(id).unwrap().inner().str_relevance(query)
    }
    */

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
    fn similar(&self, query: &str, limit: usize) -> Vec<EngineItem>;
}
