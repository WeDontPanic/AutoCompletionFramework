pub mod item;

pub use item::Item;
use serde::{Deserialize, Serialize};

use super::{IndexItem, KanjiReadingAlign, SuggestionIndex};
use crate::relevance::item::EngineItem;
use order_struct::{float_ord::FloatOrd, OrderBy};
use priority_container::{PrioContainer, PrioContainerMax};
use qp_trie::{wrapper::BString, Trie};
use std::collections::HashSet;

/// Japanese suggestion index
#[derive(Serialize, Deserialize)]
pub struct JapaneseIndex {
    pub trie: Trie<BString, Vec<u32>>,
    pub items: Vec<Item>,
    kanji_align: Trie<BString, Vec<u32>>,
}

pub struct InsertItem {
    item: Item,
    kanji_aligns: Vec<String>,
    normal_kana: Option<String>,
}

impl InsertItem {
    #[inline]
    pub fn new(item: Item, kanji_aligns: Vec<String>) -> Self {
        Self {
            item,
            kanji_aligns,
            normal_kana: None,
        }
    }

    /// Set the insert item's normal kana.
    #[inline]
    pub fn set_normal_kana(&mut self, normal_kana: Option<String>) {
        self.normal_kana = normal_kana;
    }
}

impl JapaneseIndex {
    /// Build a new JapaneseIndex
    pub fn new(items: Vec<InsertItem>) -> Self {
        let mut trie = Trie::new();
        let mut kanji_align = Trie::new();

        let mut index_item = Vec::with_capacity(items.len());

        for (pos, iitem) in items.iter().enumerate() {
            let id = pos as u32;
            let kanji_aligns = &iitem.kanji_aligns;
            let item = iitem.item.clone();

            insert_or_update(&mut trie, &item.kana, id);

            if let Some(ref kanji) = item.kanji {
                insert_or_update(&mut trie, kanji, id);
            }

            for alt in &item.alternative {
                insert_or_update(&mut trie, &alt, id);
            }

            for item in kanji_aligns {
                insert_or_update(&mut kanji_align, &item, id);
            }

            if let Some(ref i) = iitem.normal_kana {
                insert_or_update(&mut trie, i, id);
            }

            index_item.push(item);
        }

        Self {
            trie,
            items: index_item,
            kanji_align,
        }
    }

    #[inline]
    pub fn get_item(&self, id: u32) -> &Item {
        &self.items[id as usize]
    }
}

impl SuggestionIndex for JapaneseIndex {
    fn predictions(&self, inp: &str, limit: usize) -> Vec<EngineItem> {
        let mut prio_container = PrioContainerMax::new_allocated(limit);
        let mut pev_dups: HashSet<&Item> = HashSet::with_capacity(limit * 2);

        let items = self.trie.iter_prefix_str(inp);
        for j in items.map(|i| i.1).flatten() {
            let word = self.get_item(*j);

            if pev_dups.contains(word) {
                continue;
            }

            prio_container.insert(OrderBy::new(word, |a, b| {
                FloatOrd(b.frequency).cmp(&FloatOrd(a.frequency)).reverse()
            }));

            pev_dups.insert(word);
        }

        // PrioContainer only yields `limit` items
        prio_container
            .into_iter()
            .map(|i| i.0.into_inner().into_engine_item())
            .collect()
    }

    fn similar_terms(&self, inp: &str, limit: usize, max_dist: u32) -> Vec<EngineItem> {
        let query_hash = match jpeudex::Hash::new(inp) {
            Some(h) => h,
            None => return vec![],
        };

        let mut out = PrioContainer::new_allocated(limit);

        let prefix = inp.char_indices().nth(1).map(|i| &inp[0..i.0]).unwrap();

        let iter = self
            .trie
            .iter_prefix_str(prefix)
            .map(|i| i.1.iter().map(|j| self.get_item(*j)))
            .flatten();

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
    fn exact(&self, inp: &str) -> Vec<EngineItem> {
        self.trie
            .get_str(inp)
            .map(|id| id.into_iter().filter_map(|i| self.get_word(*i)).collect())
            .unwrap_or_default()
    }

    #[inline]
    fn get_word(&self, id: u32) -> Option<EngineItem> {
        self.items.get(id as usize).map(|i| i.into_engine_item())
    }

    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl KanjiReadingAlign for JapaneseIndex {
    fn align_reading(&self, query: &str) -> Vec<EngineItem> {
        let mut out = HashSet::new();
        for i in self.kanji_align.subtrie_str(query) {
            for word in i.1 {
                out.insert(self.get_item(*word).into_engine_item());
            }
        }
        out.into_iter().collect()
    }
}

fn insert_or_update(trie: &mut Trie<BString, Vec<u32>>, item: &str, id: u32) {
    if let Some(v) = trie.get_mut_str(item) {
        v.push(id);
    } else {
        trie.insert_str(item, vec![id]);
    }
}

// Format inp
pub fn jp_format(inp: &str) -> String {
    let mut out = inp.to_string();
    let to_replace = &[
        "(", ")", ".", ",", "/", "[", "]", "?", "!", "{", "}", "、", "。", "・",
    ];
    for tr in to_replace {
        out = out.replace(tr, "");
    }
    out.to_lowercase()
}
