use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qi_lang::eval::Evaluator;
use qi_lang::parser::Parser;

fn bench_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_calls");

    // SmallVec最適化の効果測定（≤4引数）
    group.bench_function("4_args", |b| {
        let code = "(defn add4 [a b c d] (+ a b c d)) (add4 1 2 3 4)";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse_all().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            for expr in &ast {
                black_box(evaluator.eval(expr).unwrap());
            }
        });
    });

    // SmallVecの境界を超える場合（6引数）
    group.bench_function("6_args", |b| {
        let code = "(defn add6 [a b c d e f] (+ a b c d e f)) (add6 1 2 3 4 5 6)";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse_all().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            for expr in &ast {
                black_box(evaluator.eval(expr).unwrap());
            }
        });
    });

    // 高階関数（map）
    group.bench_function("map_small_list", |b| {
        let code = "(map (fn [x] (* x 2)) [1 2 3 4])";
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_json_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_processing");

    // JSON stringify（with_capacity最適化の効果測定）
    group.bench_function("stringify_small", |b| {
        let code = r#"(json/stringify {:name "test" :value 42 :active true})"#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.bench_function("stringify_large", |b| {
        let code = r#"(json/stringify {:a 1 :b 2 :c 3 :d 4 :e 5 :f 6 :g 7 :h 8 :i 9 :j 10})"#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    // JSON parse
    group.bench_function("parse_small", |b| {
        let code = r#"(json/parse "{\"name\":\"test\",\"value\":42}")"#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_csv_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_processing");

    // CSV parse（with_capacity最適化の効果測定）
    group.bench_function("parse_small", |b| {
        let code = r#"(csv/parse "a,b,c\n1,2,3\n4,5,6")"#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    // CSV stringify
    group.bench_function("stringify_small", |b| {
        let code = r#"(csv/stringify [["a" "b" "c"] ["1" "2" "3"] ["4" "5" "6"]])"#;
        let mut parser = Parser::new(code).unwrap();
        let ast = parser.parse().unwrap();

        b.iter(|| {
            let evaluator = Evaluator::new();
            black_box(evaluator.eval(&ast).unwrap());
        });
    });

    group.finish();
}

fn bench_sqlite(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqlite");

    // SQLiteステートメントキャッシュの効果測定
    group.bench_function("repeated_query", |b| {
        let code = r#"
(do
  (def conn (db/connect "sqlite::memory:"))
  (db/exec conn "CREATE TABLE test (id INTEGER, name TEXT)")
  (db/exec conn "INSERT INTO test VALUES (1, 'Alice')")
  (db/exec conn "INSERT INTO test VALUES (2, 'Bob')")
  (db/exec conn "INSERT INTO test VALUES (3, 'Charlie')")

  ;; 同じクエリを繰り返し実行（キャッシュの効果を確認）
  (db/query conn "SELECT * FROM test WHERE id = ?" [1])
  (db/query conn "SELECT * FROM test WHERE id = ?" [2])
  (db/query conn "SELECT * FROM test WHERE id = ?" [3])
  (db/query conn "SELECT * FROM test WHERE id = ?" [1])
  (db/query conn "SELECT * FROM test WHERE id = ?" [2]))
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
    bench_function_calls,
    bench_json_processing,
    bench_csv_processing,
    bench_sqlite
);
criterion_main!(benches);
