use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qi_lang::{eval::Evaluator, parser::Parser};

/// 大規模データセット用のベンチマーク
///
/// 10,000-100,000要素のリスト処理で、Phase 6の最適化効果を測定

fn bench_large_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");

    group.bench_function("map_10k", |b| {
        let code = "(map (fn [x] (* x 2)) (range 10000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.bench_function("map_50k", |b| {
        let code = "(map (fn [x] (* x 2)) (range 50000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_large_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");

    group.bench_function("filter_10k", |b| {
        let code = "(filter (fn [x] (> x 5000)) (range 10000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.bench_function("filter_50k", |b| {
        let code = "(filter (fn [x] (> x 25000)) (range 50000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_large_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");

    group.bench_function("reduce_10k", |b| {
        let code = "(reduce + 0 (range 10000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.bench_function("reduce_50k", |b| {
        let code = "(reduce + 0 (range 50000))";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_large_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");

    group.bench_function("pipeline_10k", |b| {
        let code = r#"
            (let [data (range 10000)]
              (data
               |> (map (fn [x] (* x 3)))
               |> (filter (fn [x] (> x 5000)))
               |> (map (fn [x] (+ x 100)))
               |> (reduce + 0)))
        "#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.bench_function("pipeline_50k", |b| {
        let code = r#"
            (let [data (range 50000)]
              (data
               |> (map (fn [x] (* x 3)))
               |> (filter (fn [x] (> x 25000)))
               |> (map (fn [x] (+ x 100)))
               |> (reduce + 0)))
        "#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_nested_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_dataset");

    group.bench_function("nested_map_1k", |b| {
        let code = r#"
            (map
              (fn [x]
                (reduce + 0
                  (map (fn [y] (* x y))
                       (range 100))))
              (range 1000))
        "#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_large_map,
    bench_large_filter,
    bench_large_reduce,
    bench_large_pipeline,
    bench_nested_operations
);
criterion_main!(benches);
