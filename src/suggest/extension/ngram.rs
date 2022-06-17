use priority_container::PrioContainerMax;

use crate::{
    index::NGIndexable,
    relevance::{item::EngineItem, RelevanceCalc},
    suggest::query::SuggestionQuery,
};

use super::{Extension, ExtensionOptions};

pub struct NGramExtension<'a> {
    pub options: ExtensionOptions,
    index: &'a dyn NGIndexable,
}

impl<'a> NGramExtension<'a> {
    pub fn new(index: &'a dyn NGIndexable) -> Self {
        let mut options = ExtensionOptions::default();
        options.threshold = 10;
        Self { options, index }
    }
}

impl<'a> Extension<'a> for NGramExtension<'a> {
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;
        let mut out = PrioContainerMax::new(self.options.limit);
        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);

        for mut item in self.index.similar(&query.query_str, self.options.limit) {
            let str_rel = item.get_relevance();
            let rel = rel_calc.calc(&item, str_rel);
            item.set_relevance(rel);
            /*
                let old = item.get_relevance();
                //let str_rel = item.inner().str_relevance(&query.query_str);
                println!("{:?}: strrel: {str_rel}", item.to_output());
                println!("{:?} {old}-> {}", item.to_output(), item.get_relevance());
            */
            out.insert(item);
        }

        let mut out = out.into_iter().map(|i| i.0).collect::<Vec<_>>();
        out.reverse();
        out
    }

    fn should_run(&self, already_found: usize, _query: &SuggestionQuery) -> bool {
        self.options.enabled && already_found < self.options.threshold
    }

    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}
