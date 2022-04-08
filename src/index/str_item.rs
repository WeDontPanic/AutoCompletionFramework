use crate::relevance::item::EngineItem;

use super::{IndexItem, Output, ToOutput};

#[derive(Clone, Debug)]
pub struct StringItem {
    word: String,
    frequency: f64,
}

impl StringItem {
    /// Create a new String Item
    pub fn new(word: String, frequency: f64) -> Self {
        Self { word, frequency }
    }
}

impl ToOutput for StringItem {
    #[inline]
    fn to_output(&self) -> Output {
        Output {
            primary: self.word.to_string(),
            secondary: None,
        }
    }
}

impl IndexItem for StringItem {
    #[inline]
    fn frequency(&self) -> f64 {
        self.frequency
    }

    #[inline]
    fn str_relevance(&self, query: &str) -> u16 {
        let query = query.to_lowercase();
        if self.word.starts_with(&query) {
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
        0
    }
}
