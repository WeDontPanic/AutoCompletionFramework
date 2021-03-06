use priority_container::PrioContainerMax;

use crate::{
    index::NGIndexable,
    relevance::{item::EngineItem, RelevanceCalc},
    suggest::query::SuggestionQuery,
};

use super::{Extension, ExtensionOptions};

pub struct NGramExtension<'a> {
    pub options: ExtensionOptions,
    pub sim_threshold: u16,
    pub query_weigth: f32,
    pub term_limit: usize,
    index: &'a dyn NGIndexable,
    pub cust_query: Option<String>,
}

impl<'a> NGramExtension<'a> {
    pub fn new(index: &'a dyn NGIndexable) -> Self {
        Self::with_sim_threshold(index, 0.45)
    }

    pub fn with_sim_threshold(index: &'a dyn NGIndexable, sim_threshold: f32) -> Self {
        let mut options = ExtensionOptions::default();
        options.threshold = 10;
        Self {
            options,
            index,
            sim_threshold: (sim_threshold * 1000.0) as u16,
            query_weigth: 0.6,
            term_limit: 2000,
            cust_query: None,
        }
    }

    pub fn get_query(&'a self, query: &'a str) -> &'a str {
        if self.cust_query.is_some() {
            return self.cust_query.as_ref().unwrap();
        }
        query
    }
}

impl<'a> Extension<'a> for NGramExtension<'a> {
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;

        let mut out = PrioContainerMax::new(self.options.limit);

        let rel_calc = RelevanceCalc::new(self.options.weights).with_total_weight(rel_weight);

        let q_str = self.get_query(&query.query_str);

        for mut item in self.index.similar(
            q_str,
            self.options.limit,
            self.query_weigth,
            self.term_limit,
        ) {
            // use previously assigned value form ngam index as string relevance
            let str_rel = item.get_relevance();
            if str_rel < self.sim_threshold {
                continue;
            }
            item.set_relevance(rel_calc.calc(&item, str_rel));
            out.insert(item);
        }

        let mut out = out.into_iter().map(|i| i.0).collect::<Vec<_>>();
        out.reverse();
        out
    }

    #[inline]
    fn should_run(&self, already_found: usize, query: &SuggestionQuery) -> bool {
        let q_str = self.get_query(&query.query_str);
        self.options.enabled
            && already_found < self.options.threshold
            && q_str.len() >= self.options.min_query_len
    }

    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}
