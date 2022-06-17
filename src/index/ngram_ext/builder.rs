use std::io::Cursor;

use vector_space_model2::{
    build::IndexBuilder,
    metadata::IndexVersion,
    traits::{Decodable, Encodable},
    DefaultMetadata, Index,
};

use super::{iter::NgramIter, NGIndex};

pub struct NGIndexBuilder<I: Decodable + Encodable> {
    builder: IndexBuilder<I>,
    n: usize,
}

impl<I: Decodable + Encodable> NGIndexBuilder<I> {
    pub fn new(n: usize) -> Self {
        let builder = IndexBuilder::<I>::new();
        Self { builder, n }
    }

    pub fn insert(&mut self, term: &str, id: I) -> bool {
        let term_len = term.chars().count();
        if term_len < self.n {
            return false;
        }

        let padded = super::padded(term, self.n);
        let terms: Vec<_> = self.split_term(&padded).collect();
        self.builder.insert_new_vec(id, &terms);

        true
    }

    pub fn build(self) -> NGIndex<I> {
        let mut buf = vec![];
        self.builder
            .build(&mut buf, DefaultMetadata::new(IndexVersion::V1))
            .unwrap();
        let index = Index::<I, DefaultMetadata>::from_reader(Cursor::new(buf)).unwrap();
        NGIndex::new(index, self.n)
    }

    fn split_term<'a>(&self, term: &'a str) -> NgramIter<'a> {
        NgramIter::new(term, self.n)
    }
}
