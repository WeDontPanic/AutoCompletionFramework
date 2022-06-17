pub mod builder;
pub mod item;

pub use item::Item;

use super::{ngram_ext::NGIndex, IndexItem, SuggestionIndex};
use crate::relevance::item::EngineItem;
use priority_container::PrioContainerMax;
use serde::{Deserialize, Serialize};
use vector_space_model2::Vector;

/// Index with basic suggestion functionality
#[derive(Serialize, Deserialize)]
pub struct NgramIndex {
    /// All Words, with the vector position as ID and frequency data
    terms: Vec<Item>,
    index: NGIndex<Vec<u32>>,
    n: usize,
}

impl NgramIndex {
    #[inline]
    pub fn get_item(&self, id: u32) -> &Item {
        &self.terms[id as usize]
    }

    #[inline]
    fn get_results<'q>(&'q self, query: &'q Vector) -> impl Iterator<Item = (Vec<u32>, f32)> + 'q {
        self.index.find(query)
    }

    #[inline]
    fn build_query(&self, s: &str) -> Option<Vector> {
        self.index.query_vec(s)
    }
}

impl SuggestionIndex for NgramIndex {
    fn predictions(&self, inp: &str, limit: usize) -> Vec<EngineItem> {
        let query = match self.build_query(inp) {
            Some(q) => q,
            None => return vec![],
        };

        let mut prio_queue = PrioContainerMax::new(limit);

        for (r_vecs, similarity) in self.get_results(&query) {
            for item in r_vecs.iter().map(|id| self.get_item(*id)) {
                let mut engine_item = item.into_engine_item();
                engine_item.set_relevance((similarity * 1000.0) as u16);
                prio_queue.insert(engine_item);
            }
        }

        let mut out: Vec<_> = prio_queue.into_iter().map(|i| i.0).collect();
        out.reverse();
        out
    }

    fn exact(&self, inp: &str) -> Vec<EngineItem> {
        let query = match self.build_query(inp) {
            Some(q) => q,
            None => return vec![],
        };
        let r = self
            .get_results(&query)
            .filter(|i| i.1 >= 0.99999)
            .map(|(i, _)| {
                i.iter()
                    .filter_map(|id| {
                        let mut word = self.get_word(*id)?;
                        word.set_relevance(1000);
                        Some(word)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten();
        r.collect()
    }

    #[inline]
    fn get_word(&self, id: u32) -> Option<EngineItem> {
        self.terms.get(id as usize).map(|i| i.into_engine_item())
    }

    #[inline]
    fn len(&self) -> usize {
        self.terms.len()
    }
}

pub fn padded(word: &str, n: usize) -> String {
    let pads = "§".repeat(n - 1);
    format!("{pads}{word}{pads}")
}

/*
fn dice(a: &Vector, b: &Vector) -> f32 {
    let overlapping_cnt = a.overlapping(b).count() as f32 * 2.0;
    overlapping_cnt / (a.dimen_count() as f32 + b.dimen_count() as f32)
}
*/
