use priority_container::PrioContainerMax;

use super::{Extension, ExtensionOptions};
use crate::{
    fast_str_diff::FastStringDist,
    index::SuggestionIndex,
    relevance::{item::EngineItem, RelevanceCalc},
    suggest::query::SuggestionQuery,
};

#[derive(Clone, Copy)]
pub struct SimilarTermsExtension<'a> {
    pub options: ExtensionOptions,
    index: &'a dyn SuggestionIndex,
    pub max_str_dist: u32,
}

impl<'a> SimilarTermsExtension<'a> {
    /// Create a new Longest-Prefix Extension
    pub fn new(index: &'a dyn SuggestionIndex, max_str_dist: u32) -> Self {
        let options = ExtensionOptions::default();
        Self {
            options,
            index,
            max_str_dist,
        }
    }
}

impl<'a> Extension<'a> for SimilarTermsExtension<'a> {
    #[inline]
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;

        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);
        let similar =
            self.index
                .similar_terms(&query.query_str, self.options.limit * 10, self.max_str_dist);

        let out = FastStringDist::new(similar, &query.query_str).assign_mut(|item, query| {
            let str_rel = item
                .inner()
                .str_relevance(query)
                .saturating_sub(item.get_relevance() * 5);
            item.set_relevance(rel_calc.calc(item, str_rel));
        });
        let mut out_pq = PrioContainerMax::new(self.options.limit);
        out_pq.extend(out);
        out_pq.into_iter().map(|i| i.0).collect()
    }

    #[inline]
    fn should_run(&self, already_found: usize, _query: &SuggestionQuery) -> bool {
        self.options.enabled && already_found < self.options.threshold
    }

    #[inline]
    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}
