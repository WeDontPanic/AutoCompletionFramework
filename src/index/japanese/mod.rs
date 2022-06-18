pub mod builder;
pub mod item;

pub use item::Item;
use ngindex::NGIndex;
use serde::{Deserialize, Serialize};

use super::{IndexItem, KanjiReadingAlign, NGIndexable, SuggestionIndex};
use crate::relevance::item::EngineItem;
use order_struct::{float_ord::FloatOrd, order_nh::OrderVal, OrderBy};
use priority_container::{PrioContainer, PrioContainerMax};
use qp_trie::{wrapper::BString, Trie};
use std::collections::HashSet;

/// Japanese suggestion index
#[derive(Serialize, Deserialize)]
pub struct JapaneseIndex {
    pub trie: Trie<BString, Vec<u32>>,
    pub items: Vec<Item>,
    kanji_align: Trie<BString, Vec<u32>>,

    ngindex: NGIndex<Vec<u32>>,
}

impl JapaneseIndex {
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
                FloatOrd(a.frequency).cmp(&FloatOrd(b.frequency))
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
        let inp_len = inp.trim().chars().count();
        if inp_len <= 1 {
            return vec![];
        }
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

impl NGIndexable for JapaneseIndex {
    fn similar(&self, query: &str, limit: usize) -> Vec<EngineItem> {
        let q_vec = match self.ngindex.make_query_vec(query) {
            Some(q) => q,
            None => return vec![],
        };

        let mut prio_queue = PrioContainerMax::new(limit);

        let res_iter = self
            .ngindex
            .find_qweight(&q_vec, 0.64)
            .map(|(id, sim)| OrderVal::new(id, FloatOrd(sim)));
        prio_queue.extend(res_iter);

        let mut out: Vec<_> = prio_queue
            .into_iter()
            .map(|i| {
                let rel = i.0.ord().0;
                i.0.into_inner()
                    .into_iter()
                    .map(|i| EngineItem::new(self.get_item(i), (rel * 1000.0) as u16))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        out.reverse();
        out
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
