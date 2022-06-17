use serde::{Deserialize, Serialize};

use crate::{
    index::{Output, ToOutput},
    relevance::item::EngineItem,
};

/// An item in the Basic index
#[derive(Serialize, Deserialize)]
pub struct Item {
    pub(crate) word_id: u32,
    pub(crate) word: String,
    pub(crate) frequency: f64,
    pub(crate) hash: Option<eudex::Hash>,
}

impl Item {
    /// Create a new index item
    #[inline]
    pub fn new(word: String, word_id: u32, frequency: f64) -> Self {
        assert!(frequency <= 1.0);
        let hash = (word.len() <= 16).then(|| eudex::Hash::new(&word));
        Self {
            word_id,
            word,
            frequency,
            hash,
        }
    }

    /// Get a reference to the index item's word.
    #[inline]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// Get the index item's frequency.
    #[inline]
    pub fn frequency(&self) -> f64 {
        self.frequency
    }
}

impl ToOutput for Item {
    #[inline]
    fn to_output(&self) -> Output {
        Output {
            primary: self.word.clone(),
            secondary: None,
        }
    }
}

impl super::super::IndexItem for Item {
    #[inline]
    fn frequency(&self) -> f64 {
        self.frequency
    }

    #[inline]
    fn str_relevance(&self, query: &str) -> u16 {
        let query = query.to_lowercase();
        if self.word.to_lowercase().starts_with(&query) {
            // Give shorter matches more priority. For exact matches (lame length) => normalized=0
            let normalized = 1.0 - (query.len() as f32 / self.word.len() as f32);
            1000 - (normalized * 1000.0) as u16
        } else {
            (strsim::normalized_levenshtein(&self.word, &query) * 100.0) as u16
        }
    }

    #[inline]
    fn into_engine_item(&self) -> EngineItem {
        EngineItem::new(self, 0)
    }

    #[inline]
    fn terms(&self) -> Vec<&String> {
        vec![&self.word]
    }

    #[inline]
    fn word_id(&self) -> u32 {
        self.word_id
    }
}
