use super::item::Item;
use super::NgramIndex;
use std::collections::HashMap;

pub struct NgramIndexBuilder {
    out_terms: Vec<Item>,
    index_str: HashMap<String, Vec<usize>>,
    n: usize,
}

impl NgramIndexBuilder {
    pub fn new(n: usize) -> Self {
        Self {
            out_terms: vec![],
            index_str: HashMap::new(),
            n,
        }
    }

    pub fn insert<S: AsRef<str>>(&mut self, terms: &[S], item: Item) {
        let pos = self.out_terms.len();
        self.out_terms.push(item);

        for term in terms {
            let term = term.as_ref().to_string();
            self.index_str.entry(term).or_default().push(pos);
        }
    }

    pub fn build(self) -> NgramIndex {
        let mut builder = super::super::ngram_ext::builder::NGIndexBuilder::<Vec<u32>>::new(self.n);

        for (term, out) in self.index_str {
            let out: Vec<_> = out.into_iter().map(|i| i as u32).collect();
            builder.insert(&term, out);
        }

        NgramIndex {
            terms: self.out_terms,
            index: builder.build(),
            n: self.n,
        }
    }
}
