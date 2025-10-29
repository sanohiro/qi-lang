use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qi_lang::{parse, Evaluator};

/// シンプルな式の評価ベンチマーク
fn bench_simple_eval(c: &mut Criterion) {
    let eval = Evaluator::new();

    c.bench_function("eval simple arithmetic", |b| {
        b.iter(|| {
            let ast = parse("(+ 1 2)").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval variable lookup", |b| {
        eval.eval(&parse("(def x 42)").unwrap()).unwrap();
        b.iter(|| {
            let ast = parse("x").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });
}

/// 関数定義と呼び出しのベンチマーク
fn bench_function_call(c: &mut Criterion) {
    let eval = Evaluator::new();

    // 単純な関数
    eval.eval(&parse("(defn add [a b] (+ a b))").unwrap())
        .unwrap();
    c.bench_function("eval function call simple", |b| {
        b.iter(|| {
            let ast = parse("(add 10 20)").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    // 再帰関数（フィボナッチ）
    eval.eval(&parse("(defn fib [n] (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2)))))").unwrap())
        .unwrap();

    let mut group = c.benchmark_group("fibonacci");
    for i in [5, 10, 15].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(i), i, |b, &n| {
            b.iter(|| {
                let ast = parse(&format!("(fib {})", n)).unwrap();
                eval.eval(black_box(&ast)).unwrap()
            });
        });
    }
    group.finish();
}

/// コレクション操作のベンチマーク
fn bench_collections(c: &mut Criterion) {
    let eval = Evaluator::new();

    c.bench_function("eval map over vector", |b| {
        b.iter(|| {
            let ast = parse("(map (fn [x] (* x 2)) [1 2 3 4 5])").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval filter vector", |b| {
        b.iter(|| {
            let ast = parse("(filter (fn [x] (> x 5)) [1 3 5 7 9 11])").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval reduce", |b| {
        b.iter(|| {
            let ast = parse("(reduce + 0 [1 2 3 4 5])").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });
}

/// パイプライン演算子のベンチマーク
fn bench_pipeline(c: &mut Criterion) {
    let eval = Evaluator::new();

    c.bench_function("eval simple pipeline", |b| {
        b.iter(|| {
            let ast = parse("[1 2 3 4 5] |> (map (fn [x] (* x 2))) |> (filter (fn [x] (> x 5)))")
                .unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval nested vs pipeline", |b| {
        b.iter(|| {
            let ast =
                parse("(filter (fn [x] (> x 5)) (map (fn [x] (* x 2)) [1 2 3 4 5]))").unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });
}

/// パターンマッチングのベンチマーク
fn bench_pattern_matching(c: &mut Criterion) {
    let eval = Evaluator::new();

    c.bench_function("eval simple match", |b| {
        b.iter(|| {
            let ast = parse(
                r#"
                (match 42
                  1 "one"
                  42 "forty-two"
                  _ "other")
            "#,
            )
            .unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval match with guard", |b| {
        b.iter(|| {
            let ast = parse(
                r#"
                (match 15
                  x (if (< x 10) "small")
                  x (if (< x 20) "medium")
                  _ "large")
            "#,
            )
            .unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });
}

/// 文字列操作のベンチマーク
fn bench_string_ops(c: &mut Criterion) {
    let eval = Evaluator::new();

    c.bench_function("eval string concat", |b| {
        b.iter(|| {
            let ast = parse(r#"(str "hello" " " "world")"#).unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });

    c.bench_function("eval string split", |b| {
        b.iter(|| {
            let ast = parse(r#"(split "," "a,b,c,d,e")"#).unwrap();
            eval.eval(black_box(&ast)).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_simple_eval,
    bench_function_call,
    bench_collections,
    bench_pipeline,
    bench_pattern_matching,
    bench_string_ops
);

criterion_main!(benches);
