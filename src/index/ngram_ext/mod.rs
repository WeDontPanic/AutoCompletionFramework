pub mod builder;
pub mod iter;

use serde::{Deserialize, Serialize};
use vector_space_model2::{index::Index, DefaultMetadata, Vector};

use self::iter::NgramIter;

#[derive(Deserialize, Serialize)]
pub struct NGIndex {
    index: Index<u32, DefaultMetadata>,
    n: usize,
}

impl NGIndex {
    pub fn new(index: Index<u32, DefaultMetadata>, n: usize) -> Self {
        Self { index, n }
    }

    pub fn query_vec(&self, query: &str) -> Option<Vector> {
        let padded_query = padded(query, self.n);
        let terms: Vec<_> = NgramIter::new(&padded_query, self.n).collect();
        self.build_vec(&terms)
    }

    fn build_vec<S: AsRef<str>>(&self, terms: &[S]) -> Option<Vector> {
        let vec = self.index.build_vector(terms, None)?;
        Some(vec)
    }

    fn light_vec_dims(&self, vec: &Vector) -> Vec<u32> {
        vec.vec_indices()
            .filter(|dim| {
                self.index
                    .get_indexer()
                    .load_term(*dim as usize)
                    .unwrap()
                    .doc_frequency()
                    < 1000
            })
            .collect()
    }

    pub fn find<'a>(&'a self, query: &'a Vector) -> impl Iterator<Item = (u32, f32)> + 'a {
        let dims = self.light_vec_dims(query);
        self.index.get_vector_store().get_all_iter(&dims).map(|i| {
            let sim = dice(i.vector(), query);
            (i.document, sim)
        })
    }
}

pub fn padded(word: &str, n: usize) -> String {
    let pads = "§".repeat(n - 1);
    format!("{pads}{word}{pads}")
}

fn dice(a: &Vector, b: &Vector) -> f32 {
    let overlapping_cnt = a.overlapping(b).count() as f32 * 2.0;
    overlapping_cnt / (a.dimen_count() as f32 + b.dimen_count() as f32)
}
