use super::{BasicIndex, Item};
use ngindex::builder::NGIndexBuilder;
use qp_trie::{wrapper::BString, Trie};

pub struct BasicIndexBuilder {
    build_ng_index: bool,
    ng_index: NGIndexBuilder<u32>,

    items: Vec<Item>,
    trie: Trie<BString, u32>,
}

impl BasicIndexBuilder {
    pub fn new() -> Self {
        Self {
            build_ng_index: false,
            // some dummy value we don't care since we don't use this
            ng_index: NGIndexBuilder::new(10),
            items: vec![],
            trie: Trie::new(),
        }
    }

    pub fn with_ngindex(n: usize) -> Self {
        Self {
            build_ng_index: true,
            ng_index: NGIndexBuilder::new(n),
            items: vec![],
            trie: Trie::new(),
        }
    }

    /// Returns Ok(u32) with the ID of the newly inserted item. Err(Item)
    // with the passed item if the term already existed
    pub fn insert(&mut self, item: Item, formatted: &str) -> Result<u32, Item> {
        let id = self.items.len() as u32;

        if self.trie.contains_key_str(formatted) {
            return Err(item);
        }

        self.items.push(item);
        self.trie.insert_str(formatted, id);

        Ok(id)
    }

    pub fn insert_ng(&mut self, formatted: &str, id: u32) {
        if !self.build_ng_index {
            return;
        }
        self.ng_index.insert(formatted, id);
    }

    pub fn build(self) -> BasicIndex {
        let ngram = self.ng_index.build();
        BasicIndex {
            trie: self.trie,
            terms: self.items,
            ngram,
        }
    }
}
