use serde::{Deserialize, Serialize};

use crate::{
    index::{Output, ToOutput},
    relevance::item::EngineItem,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub word_id: u32,
    pub kana: String,
    pub kanji: Option<String>,
    pub alternative: Vec<String>,

    pub frequency: f64,

    // kana hashes
    pub(crate) hash: Option<jpeudex::Hash>,
}

impl Item {
    /// Create a new Item
    #[inline]
    pub fn new(
        word_id: u32,
        kana: String,
        kanji: Option<String>,
        alternative: Vec<String>,
        frequency: f64,
    ) -> Self {
        assert!(frequency <= 1.0);
        let hash = jpeudex::Hash::new(&kana);
        Self {
            word_id,
            kana,
            kanji,
            alternative,
            frequency,
            hash,
        }
    }
}

impl ToOutput for Item {
    #[inline]
    fn to_output(&self) -> Output {
        Output {
            primary: self.kana.clone(),
            secondary: self.kanji.clone(),
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
        fn freq(word: &str, query: &str) -> u16 {
            if word.starts_with(&query) {
                let query_len: usize = query.chars().count();
                let word_len: usize = word.chars().count();

                let normalized = (1.0 - (query_len as f32 / word_len as f32)) * 1000.0;
                return (1000.0 - normalized) as u16;
            } else {
                return (strsim::normalized_levenshtein(&word, &query) * 1000.0) as u16;
            }
        }

        let kana_sim = freq(&self.kana, query);
        let kanji_sim = self
            .kanji
            .as_ref()
            .map(|i| freq(i, query))
            .max(self.alternative.iter().map(|i| freq(i, query)).max())
            .unwrap_or(0);
        kana_sim.max(kanji_sim)
    }

    #[inline]
    fn into_engine_item(&self) -> EngineItem {
        EngineItem::new(self, 0)
    }

    fn terms(&self) -> Vec<&String> {
        let mut out = vec![&self.kana];
        if let Some(k) = &self.kanji {
            out.push(k);
        }

        out
    }

    #[inline]
    fn word_id(&self) -> u32 {
        self.word_id
    }
}

impl std::hash::Hash for Item {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kana.hash(state);
        self.kanji.hash(state);
        self.alternative.hash(state);
    }
}

impl PartialEq for Item {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.kana == other.kana
            && self.kanji == other.kanji
            && self.alternative == other.alternative
    }
}

impl Eq for Item {}
