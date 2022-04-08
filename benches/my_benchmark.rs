use std::{fs::File, io::BufReader};

use autocompletion::{
    index::{basic::BasicIndex, japanese::JapaneseIndex, SuggestionIndex},
    suggest::{query::SuggestionQuery, task::SuggestionTask},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn get_jp_index() -> JapaneseIndex {
    bincode::deserialize_from(BufReader::new(File::open("./new_jp_index").unwrap())).unwrap()
}

fn get_index() -> BasicIndex {
    bincode::deserialize_from(BufReader::new(File::open("./index").unwrap())).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let engine = get_index();

    /*
    c.bench_function("load index", |b| {
        b.iter(|| {
            get_index();
            get_jp_index();
        })
    });
    */

    c.bench_function("simple query 2", |b| {
        let term = "musi";
        b.iter(|| {
            let mut task = SuggestionTask::new(30);
            task.add_query(SuggestionQuery::new(&engine, black_box(term)));
            let res = task.search();
            assert!(res.len() > 0);
        })
    });

    c.bench_function("simple query", |b| {
        let term = "to ";
        b.iter(|| {
            let mut task = SuggestionTask::new(30);
            task.add_query(SuggestionQuery::new(&engine, black_box(term)));
            let res = task.search();
            assert!(res.len() > 0);
        })
    });

    c.bench_function("very short", |b| {
        let term = "t";
        b.iter(|| {
            let mut task = SuggestionTask::new(30);
            let query = SuggestionQuery::new(&engine, black_box(term));
            task.add_query(query);
            let res = task.search();
            assert!(res.len() > 0);
        })
    });

    c.bench_function("many queries", |b| {
        let terms = &["morn", "hom", "hen", "fl", "t", "sev", "to"];
        b.iter(|| {
            let mut task = SuggestionTask::new(30);
            for term in terms {
                task.add_query(SuggestionQuery::new(&engine, black_box(term)));
            }
            let res = task.search();
            assert!(res.len() > 0);
        })
    });

    let jp_engine = get_jp_index();
    c.bench_function("similar terms", |b| {
        b.iter(|| {
            let _ = jp_engine.similar_terms("あおそら", 30, 10000);
            let _ = engine.similar_terms("homevork", 30, 10000);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
