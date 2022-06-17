pub mod builder;
pub mod iter;

use serde::{Deserialize, Serialize};
use vector_space_model2::{index::Index, traits::Decodable, DefaultMetadata, Vector};

use self::iter::NgramIter;

#[derive(Deserialize, Serialize)]
pub struct NGIndex<I: Decodable> {
    pub(crate) index: Index<I, DefaultMetadata>,
    n: usize,
}

impl<I: Decodable> NGIndex<I> {
    pub fn new(index: Index<I, DefaultMetadata>, n: usize) -> Self {
        Self { index, n }
    }

    pub fn query_vec(&self, query: &str) -> Option<Vector> {
        let padded_query = padded(query, self.n - 1);
        let terms: Vec<_> = NgramIter::new(&padded_query, self.n).collect();
        self.build_vec(&terms)
    }

    /// Searches in the index with the given query and returns an iterator over the results with the relevance, in random order.
    pub fn find<'a>(&'a self, query: &'a Vector) -> impl Iterator<Item = (I, f32)> + 'a {
        let dims = self.light_vec_dims(query);
        self.index.get_vector_store().get_all_iter(&dims).map(|i| {
            let sim = dice(query, i.vector());
            (i.document, sim)
        })
    }

    /// Searches in the index with the given query and returns an iterator over the results with the relevance, in random order.
    /// Weigths the Vector lengths with the given value `w`
    /// w = 1.0 -> query's length is being used only
    /// w = 0.5 -> query's and results's length are equally important
    /// w = 0.0 -> results's length is being used only.
    pub fn find_qweight<'a>(
        &'a self,
        query: &'a Vector,
        w: f32,
    ) -> impl Iterator<Item = (I, f32)> + 'a {
        let dims = self.light_vec_dims(query);
        self.index
            .get_vector_store()
            .get_all_iter(&dims)
            .map(move |i| {
                let sim = dice_weighted(query, i.vector(), w);
                (i.document, sim)
            })
    }

    #[inline]
    fn build_vec<S: AsRef<str>>(&self, terms: &[S]) -> Option<Vector> {
        Some(self.index.build_vector(terms, None)?)
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
}

#[inline]
pub fn padded(word: &str, n: usize) -> String {
    let pads = "§".repeat(n);
    format!("{pads}{word}{pads}")
}

#[inline]
fn dice(a: &Vector, b: &Vector) -> f32 {
    let overlapping_cnt = a.overlapping(b).count() as f32 * 2.0;
    overlapping_cnt / ((a.dimen_count() as f32) + (b.dimen_count() as f32))
}

/// Calculates the `dice` similarity using a weight score to allow giving a custom
/// Weight distribution of the vector lengths.
/// w = 1.0 -> `a`'s length is being used only
/// w = 0.5 -> `a`'s and `b`'s length are equally important (same as [`dice`])
/// w = 0.0 -> `b`'s length is being used only.
#[inline]
fn dice_weighted(a: &Vector, b: &Vector, w: f32) -> f32 {
    let overlapping_cnt = a.overlapping(b).count() as f32 * 2.0;
    let a_len = a.dimen_count() as f32;
    let b_len = b.dimen_count() as f32;
    let a_mult = w * 2.0;
    let b_mult = (1.0 - w) * 2.0;
    let nenner = (a_len * a_mult) + (b_len * b_mult);
    overlapping_cnt / nenner
}
