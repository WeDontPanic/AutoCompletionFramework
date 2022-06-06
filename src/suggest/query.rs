use priority_container::{UniquePrioContainerMax};

use crate::{
    fast_str_diff::FastStringDist,
    index::SuggestionIndex,
    relevance::{item::EngineItem, RelevanceCalc, RelevanceWeights},
};

use super::extension::Extension;

pub struct SuggestionQuery<'index, 'ext> {
    /// Index to use for the search
    index: &'index dyn SuggestionIndex,
    /// Query
    pub query_str: String,
    pub weights: RelevanceWeights,
    /// Max items allowed to be already found in order for this
    /// Query to run
    pub threshold: usize,
    /// Additional extensions for the query
    extensions: Vec<Box<dyn Extension<'index> + 'ext>>,
}

impl<'index, 'ext> SuggestionQuery<'index, 'ext> {
    /// Create a new suggestion query
    pub fn new<S: ToString>(index: &'index dyn SuggestionIndex, query_str: S) -> Self {
        Self {
            index,
            query_str: query_str.to_string(),
            weights: RelevanceWeights::default(),
            threshold: 0,
            extensions: vec![],
        }
    }

    /// Adds an extension to the query
    pub fn add_extension<E: Extension<'index> + 'ext>(&mut self, extension: E) {
        self.extensions.push(Box::new(extension));
    }

    /// Executes the query
    pub fn search(&self, limit: usize) -> Vec<EngineItem<'index>> {
        let predictions = self.index.predictions(&self.query_str, limit);
        let mut pred_len = predictions.len();

        let pred_ordered = self.order_items(predictions, RelevanceCalc::new(self.weights));

        let mut queue = UniquePrioContainerMax::new(limit);
        queue.extend(pred_ordered);

        for extension in &self.extensions {
            if !extension.should_run(pred_len, &self) {
                continue;
            }

            let ext_res = extension.run(&self, self.weights.total_weight);
            pred_len += ext_res.len();
            queue.extend(ext_res);
        }

        queue.into_iter().collect::<Vec<_>>()
    }

    pub fn order_items<'a>(
        &self,
        inp: Vec<EngineItem<'a>>,
        rel_calc: RelevanceCalc,
    ) -> Vec<EngineItem<'a>> {
        FastStringDist::new(inp, &self.query_str).assign_mut(|item, query| {
            let str_rel = item.inner().str_relevance(query);
            item.set_relevance(rel_calc.calc(item, str_rel));
        })
    }
}
