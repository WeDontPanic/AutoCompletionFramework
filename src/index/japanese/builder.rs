use super::{Item, JapaneseIndex};
use ngindex::{builder::NGIndexBuilder, NGIndex};
use qp_trie::{wrapper::BString, Trie};
use romaji::RomajiExt;
use std::collections::{HashMap, HashSet};

/// Helper to build a new japanese autocompletion index
pub struct JpIndexBulider {
    build_ng_index: bool,
    n: usize,
    ng_map: HashMap<String, HashSet<u32>>,

    trie: Trie<BString, Vec<u32>>,
    items: Vec<Item>,
    kanji_align: Trie<BString, Vec<u32>>,
}

impl JpIndexBulider {
    pub fn new() -> Self {
        Self {
            build_ng_index: false,
            n: 0,
            ng_map: HashMap::new(),

            trie: Trie::new(),
            items: vec![],
            kanji_align: Trie::new(),
        }
    }

    pub fn with_ngindex(n: usize) -> Self {
        Self {
            build_ng_index: true,
            n,
            ng_map: HashMap::new(),

            trie: Trie::new(),
            items: vec![],
            kanji_align: Trie::new(),
        }
    }

    /// Adds an item to the new index. Uses the items readings as keys.
    /// Does not index kanji reading align and normal_kana since `item` doesn't hold that data
    /// Returns the id of the new item
    pub fn add_item(&mut self, item: Item) -> u32 {
        let id = self.items.len() as u32;

        self.insert_trie(&item.kana, id);

        if let Some(ref kanji) = item.kanji {
            self.insert_trie(kanji, id);
        }

        for alt in &item.alternative {
            self.insert_trie(alt, id);
        }

        self.items.push(item);
        id
    }

    /// Inserts `item` into the new index and uses all strings in `readings` to map to this item.
    /// Returns the id of the new item
    pub fn insert<S: AsRef<str>>(&mut self, readings: &[S], item: Item) -> u32 {
        let id = self.items.len() as u32;
        self.items.push(item);

        for reading in readings {
            self.insert_trie(reading.as_ref(), id);
        }

        id
    }

    /// Adds readings that'll map to the item with the given ID
    pub fn add_readings<S: AsRef<str>>(&mut self, readings: &[S], id: u32) {
        for reading in readings {
            self.insert_trie(reading.as_ref(), id);
        }
    }

    /// Inserts all readings into the ngram index and maps to the item with `id`
    pub fn insert_ng<S: AsRef<str>>(&mut self, readings: &[S], id: u32) {
        if !self.build_ng_index {
            return;
        }

        for reading in readings.iter().map(|i| i.as_ref()) {
            let reading = reading.to_romaji();
            self.ng_map.entry(reading).or_default().insert(id);
        }
    }

    pub fn insert_kalign<S: AsRef<str>>(&mut self, readings: &[S], id: u32) {
        for reading in readings {
            insert_or_update(&mut self.kanji_align, reading.as_ref(), id);
        }
    }

    /// Create a JapaneseIndex out of the builder
    pub fn build(self) -> JapaneseIndex {
        let mut ngindex = NGIndex::default();
        if self.build_ng_index {
            ngindex = Self::build_ngindex(self.n, self.ng_map);
        }

        JapaneseIndex {
            trie: self.trie,
            items: self.items,
            kanji_align: self.kanji_align,
            ngindex,
        }
    }

    fn build_ngindex(n: usize, ng_map: HashMap<String, HashSet<u32>>) -> NGIndex<Vec<u32>> {
        let mut builder = NGIndexBuilder::<Vec<u32>>::new(n);
        for (term, ids) in ng_map {
            let ids = ids.into_iter().collect();
            builder.insert(&term, ids);
        }
        builder.build()
    }

    fn insert_trie(&mut self, reading: &str, id: u32) {
        insert_or_update(&mut self.trie, reading, id);
    }
}

fn insert_or_update(trie: &mut Trie<BString, Vec<u32>>, item: &str, id: u32) {
    if let Some(v) = trie.get_mut_str(item) {
        v.push(id);
    } else {
        trie.insert_str(item, vec![id]);
    }
}
