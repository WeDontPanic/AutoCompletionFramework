#![allow(unused)]

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    ops::Add,
    time::Instant,
};

use autocompletion::{
    index::{
        self,
        basic::{basic_format, BasicIndex},
        japanese::JapaneseIndex,
        str_item::StringItem,
        IndexItem, KanjiReadingAlign, SuggestionIndex,
    },
    suggest::{
        extension::{
            custom::CustomExtension, kanji_align::KanjiAlignExtension,
            longest_prefix::LongestPrefixExtension, similar_terms::SimilarTermsExtension,
        },
        query::SuggestionQuery,
        task::SuggestionTask,
    },
};

pub fn main() {
    //let index = load_jp();
    let index: BasicIndex =
        bincode::deserialize_from(BufReader::new(File::open("./new_word_de-DE").unwrap())).unwrap();
    //let index = build();
    //bincode::serialize_into(BufWriter::new(File::create("index").unwrap()), &index).unwrap();

    println!("Index loaded ({})", index.len());

    let mut s = String::new();
    loop {
        std::io::stdin().read_line(&mut s).unwrap();
        s = s.replace("\n", "");

        search(&index, &s);

        s.clear();
    }
}

fn search<T: SuggestionIndex + 'static>(engine: &T, query: &str) {
    let query = query.to_lowercase();
    let start = Instant::now();
    let mut task = SuggestionTask::new(30).debug();

    let mut query = SuggestionQuery::new(engine, query);
    query.weights.str_weight = 0.1;
    query.weights.freq_weight = 1.9;

    /*
    query.add_extension(SimilarTermsExtension::new(engine.get_index()));
    */
    let mut lpe = LongestPrefixExtension::new(engine, 0, 5);
    lpe.options.weights.freq_weight = 0.3;
    lpe.options.weights.str_weight = 0.8;
    //query.add_extension(lpe);

    let mut ste = SimilarTermsExtension::new(engine, 7);
    ste.options.threshold = 10;
    query.add_extension(ste);

    let mut ste = SimilarTermsExtension::new(engine, 5);
    ste.options.weights.freq_weight = 0.01;
    ste.options.weights.str_weight = 1.99;
    query.add_extension(ste);

    task.add_query(query);
    let completions = task.search();
    let end = start.elapsed();

    println!("{:#?}", completions);
    println!("{:#?}", end);
    println!("aaa");
}

pub fn build() -> BasicIndex {
    let terms = all_docs();
    let freq_data = load_freq_list();

    let items = terms
        .into_iter()
        .map(|i| {
            let freq = freq_data.get(&i).unwrap_or(&0.0);
            index::basic::Item::new(i, 0, *freq)
        })
        .collect::<Vec<_>>();

    BasicIndex::new(items, basic_format)
}

fn load_freq_list() -> HashMap<String, f64> {
    let mut out = HashMap::new();

    let reader = BufReader::new(File::open("./1_2_all_freq.txt").unwrap());
    let mut sum = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let mut split = line.split('\t');
        let _ = split.next();
        let word = split.next().unwrap();
        let freq: u32 = split.skip(1).next().unwrap().trim().parse().unwrap();
        let word = basic_format(word).to_lowercase();
        sum += freq;
        out.insert(word, freq as f64);
    }

    // normalize
    out.iter_mut().for_each(|(_, v)| {
        *v /= sum as f64;
    });

    out
}

fn all_docs() -> Vec<String> {
    let mut terms = vec![];

    //let file = File::open("./de-DE").unwrap();
    let file = File::open("./en-US").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        terms.push(line);
    }
    terms
}

pub fn load_jp() -> JapaneseIndex {
    let file = File::open("./new_jp_index").unwrap();
    bincode::deserialize_from(BufReader::new(file)).unwrap()
}
