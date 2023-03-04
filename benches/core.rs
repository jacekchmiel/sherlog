use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use sherlog::Sherlog;

const TEXT: &str = include_str!("benchmark.log");
const TEXT_MULTIPLICATION_FACTOR: usize = 10;
const LINES_ON_SCREEN: usize = 80;
const SCROLL_LEN: usize = 200;

fn filter_benchmark(c: &mut Criterion) {
    let text_multiplied: String = TEXT.repeat(TEXT_MULTIPLICATION_FACTOR);

    c.bench_function("filter request-all-lines", |b| {
        let mut sherlog = Sherlog::new(&text_multiplied);
        sherlog.filters = vec![Regex::new("kernel").unwrap().into()];

        b.iter(|| black_box(sherlog.get_lines(0, None)));
    });

    c.bench_function(
        &format!("filter scroll-{SCROLL_LEN}-lines-from-middle"),
        |b| {
            let mut sherlog = Sherlog::new(&text_multiplied);
            sherlog.filters = vec![Regex::new("kernel").unwrap().into()];
            let line_cnt = sherlog.line_count();
            let start = line_cnt / 2;
            let end = start + 200;

            b.iter(|| {
                for i in start..end {
                    black_box(sherlog.get_lines(i, Some(LINES_ON_SCREEN)));
                }
            });
        },
    );
}

fn unprocessed_benchmark(c: &mut Criterion) {
    let text_multiplied: String = TEXT.repeat(TEXT_MULTIPLICATION_FACTOR);

    c.bench_function("unprocessed request-all-lines", |b| {
        let mut sherlog = Sherlog::new(&text_multiplied);
        sherlog.filters = vec![];

        b.iter(|| black_box(sherlog.get_lines(0, None)));
    });

    c.bench_function(
        &format!("unprocessed scroll-{SCROLL_LEN}-lines-from-middle"),
        |b| {
            let mut sherlog = Sherlog::new(&text_multiplied);
            sherlog.filters = vec![];

            let line_cnt = sherlog.line_count();
            const LINES_ON_SCREEN: usize = 80;
            let start = line_cnt / 2;
            let end = start + 200;

            b.iter(|| {
                for i in start..end {
                    black_box(sherlog.get_lines(i, Some(LINES_ON_SCREEN)));
                }
            });
        },
    );
}

criterion_group!(benches, filter_benchmark, unprocessed_benchmark);
criterion_main!(benches);
