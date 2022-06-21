use priority_container::PrioContainerMax;

use super::{Extension, ExtensionOptions};
use crate::{
    index::KanjiReadingAlign,
    relevance::{item::EngineItem, RelevanceCalc},
    suggest::query::SuggestionQuery,
};

#[derive(Clone, Copy)]
pub struct KanjiAlignExtension<'a> {
    pub options: ExtensionOptions,
    index: &'a dyn KanjiReadingAlign,
}

impl<'a> KanjiAlignExtension<'a> {
    /// Create a new Longest-Prefix Extension
    pub fn new(index: &'a dyn KanjiReadingAlign) -> Self {
        let mut options = ExtensionOptions::default();
        options.weights.freq_weight = 0.01;
        Self { options, index }
    }
}

impl<'a> Extension<'a> for KanjiAlignExtension<'a> {
    #[inline]
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;

        let mut out = PrioContainerMax::new(self.options.limit);

        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);
        for mut item in self.index.align_reading(&query.query_str) {
            //item.set_relevance((item.inner().frequency() * 1000.0) as u16);
            let str_rel = item.inner().str_relevance(&query.query_str);
            let rel = rel_calc.calc(&item, str_rel);
            item.set_relevance(rel);
            out.insert(item);
        }

        let out = out.into_iter().map(|i| i.0).collect::<Vec<_>>();
        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);
        query.order_items(out, rel_calc)
    }

    #[inline]
    fn should_run(&self, already_found: usize, query: &SuggestionQuery) -> bool {
        self.options.enabled
            && already_found < self.options.threshold
            && query.len() >= self.options.min_query_len
    }

    #[inline]
    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}
