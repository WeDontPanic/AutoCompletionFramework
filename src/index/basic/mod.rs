pub mod item;

pub use item::Item;
use order_struct::{float_ord::FloatOrd, OrderBy};

use super::{IndexItem, SuggestionIndex};
use crate::relevance::item::EngineItem;
use priority_container::PrioContainer;
use qp_trie::{wrapper::BString, Trie};
use serde::{Deserialize, Serialize};

/// Index with basic suggestion functionality
#[derive(Serialize, Deserialize)]
pub struct BasicIndex {
    /// Prefix tree to quickly find possible suggestion. The trees value is the ID/Position
    /// of the word in the `terms` vector
    trie: Trie<BString, u32>,
    /// All Words, with the vector position as ID and frequency data
    terms: Vec<Item>,
}

impl BasicIndex {
    /// Create a new index
    pub fn new<F>(items: Vec<Item>, format: F) -> Self
    where
        F: Fn(&str) -> String,
    {
        let mut trie = Trie::new();

        for (pos, item) in items.iter().enumerate() {
            let formatted = format(&item.word()).to_lowercase();
            trie.insert_str(&formatted, pos as u32);
        }

        Self { trie, terms: items }
    }

    /// Inserts a new item into the Index
    pub fn insert<F>(&mut self, item: Item, format: F)
    where
        F: Fn(&str) -> String,
    {
        let formatted = format(&item.word()).to_lowercase();
        self.trie.insert_str(&formatted, self.terms.len() as u32);
        self.terms.push(item);
    }

    /// Returns a raw Item
    #[inline]
    fn get_item(&self, id: u32) -> &Item {
        &self.terms[id as usize]
    }
}

impl SuggestionIndex for BasicIndex {
    fn predictions(&self, inp: &str, limit: usize) -> Vec<EngineItem> {
        let mut prio_container = PrioContainer::new_allocated(limit);

        let items = self.trie.iter_prefix_str(inp).map(|i| {
            OrderBy::new(self.get_item(*i.1), |a, b| {
                FloatOrd(a.frequency())
                    .cmp(&FloatOrd(b.frequency()))
                    .reverse()
            })
        });

        prio_container.extend(items);

        // PrioContainer only yields `limit` items
        prio_container
            .into_iter()
            .map(|i| i.into_inner().into_engine_item())
            .collect()
    }

    #[inline]
    fn exact(&self, inp: &str) -> Vec<EngineItem> {
        let word = match self.trie.get_str(inp).and_then(|i| self.get_word(*i)) {
            Some(st) => st,
            None => return vec![],
        };
        vec![word]
    }

    fn similar_terms(&self, inp: &str, limit: usize, max_dist: u32) -> Vec<EngineItem> {
        if inp.len() > 16 {
            // can't build proper hashes with len() > 16
            return vec![];
        }

        let query_hash = eudex::Hash::new(inp);
        let mut out = PrioContainer::new_allocated(limit);

        let prefix = inp.char_indices().nth(1).map(|i| &inp[0..i.0]).unwrap();

        let iter = self
            .trie
            .iter_prefix_str(prefix)
            .map(|i| self.get_item(*i.1));

        for term in iter {
            let hash = match &term.hash {
                Some(h) => *h,
                None => continue,
            };

            let mut engine_item = term.into_engine_item();
            let dist = (query_hash - hash).dist();
            if dist > max_dist {
                continue;
            }
            engine_item.set_relevance(dist as u16);
            out.insert(engine_item);
        }

        out.into_iter().collect()
    }

    #[inline]
    fn get_word(&self, id: u32) -> Option<EngineItem> {
        Some(self.terms.get(id as usize)?.into_engine_item())
    }

    #[inline]
    fn len(&self) -> usize {
        self.terms.len()
    }
}

// Basic input formatting helper
pub fn basic_format(inp: &str) -> String {
    let mut out = inp.to_string();
    let to_replace = &[
        "(", ")", ".", ",", "/", "[", "]", "?", "{", "}", "、", "。", "・",
    ];
    for tr in to_replace {
        out = out.replace(tr, "");
    }
    out.to_lowercase()
}
