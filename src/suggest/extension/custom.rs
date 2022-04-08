use super::{Extension, ExtensionOptions};
use crate::{index::SuggestionIndex, relevance::item::EngineItem, suggest::query::SuggestionQuery};

pub struct CustomExtension<'a, F, A> {
    pub options: ExtensionOptions,
    index: &'a dyn SuggestionIndex,
    cst_fn: F,
    run_fn: A,
}

impl<'a, F, A> CustomExtension<'a, F, A>
where
    F: Fn(&SuggestionQuery, &dyn SuggestionIndex, f64) -> Vec<EngineItem<'a>>,
    A: Fn(usize, &SuggestionQuery) -> bool,
{
    /// Create a new custom extension
    pub fn new(index: &'a dyn SuggestionIndex, cst_fn: F, run_fn: A) -> Self {
        let options = ExtensionOptions::default();
        Self {
            index,
            options,
            cst_fn,
            run_fn,
        }
    }
}

impl<'a, F, A> Extension<'a> for CustomExtension<'a, F, A>
where
    F: Fn(&SuggestionQuery, &dyn SuggestionIndex, f64) -> Vec<EngineItem<'a>>,
    A: Fn(usize, &SuggestionQuery) -> bool,
{
    #[inline]
    fn run(&self, query: &SuggestionQuery, rel_weight: f64) -> Vec<EngineItem<'a>> {
        let rel_weight = rel_weight * self.options.weights.total_weight;
        (self.cst_fn)(query, self.index, rel_weight)
    }

    #[inline]
    fn should_run(&self, already_found: usize, query: &SuggestionQuery) -> bool {
        (self.run_fn)(already_found, query)
    }

    #[inline]
    fn get_options(&self) -> &ExtensionOptions {
        &self.options
    }
}
