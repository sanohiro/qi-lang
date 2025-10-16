use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qi_lang::parser::Parser;

/// 小規模コード（数十行）- 基本的な関数定義
fn small_code() -> &'static str {
    r#"
(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(defn fibonacci [n]
  (if (<= n 1)
    n
    (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(def result (factorial 10))
(def fib-result (fibonacci 10))
"#
}

/// 中規模コード（数百行）- 複雑な関数、パターンマッチ、パイプライン
fn medium_code() -> &'static str {
    r#"
(module data-processing)

(defn process-user [user]
  (match user
    {:name name :age age :email email} ->
      (if (>= age 18)
        {:name name :age age :email email :status "adult"}
        {:name name :age age :email email :status "minor"})
    _ -> {:error "Invalid user"}))

(defn filter-adults [users]
  (users
   |> (filter (fn [u] (>= (get u :age) 18)))
   |> (map (fn [u] (assoc u :status "verified")))))

(defn calculate-stats [numbers]
  (let [sum (reduce + 0 numbers)
        cnt (count numbers)
        mean (/ sum cnt)
        squares (map (fn [x] (* (- x mean) (- x mean))) numbers)
        variance (/ (reduce + 0 squares) cnt)]
    {:mean mean :variance variance}))

(defn process-batch [items batch-size]
  (loop [remaining items
         batches []]
    (if (empty? remaining)
      batches
      (let [batch (take batch-size remaining)
            rest-items (drop batch-size remaining)]
        (recur rest-items (conj batches batch))))))

(defn safe-divide [a b]
  (try
    (/ a b)))

(mac when [test & body]
  `(if ,test (do ,@body) nil))

(defn validate-email [email]
  (match email
    s when (> (count s) 3) -> {:valid true :email s}
    _ -> {:valid false}))

(defn sum-range [start end]
  (loop [i start
         acc 0]
    (if (> i end)
      acc
      (recur (+ i 1) (+ acc i)))))

(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(defn fibonacci [n]
  (if (<= n 1)
    n
    (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(defn map-transform [coll fn]
  (coll |> (map fn) |> (filter (fn [x] (not (nil? x))))))

(export process-user filter-adults
        calculate-stats process-batch safe-divide
        validate-email sum-range factorial fibonacci map-transform)
"#
}

/// 大規模コード（数千行相当）- 実践的なモジュール
fn large_code() -> String {
    let mut code = String::with_capacity(20000);

    code.push_str("(module web-framework)\n\n");

    // 50個のシンプルな関数定義を生成
    for i in 0..50 {
        code.push_str(&format!(
            r#"
(defn handler-{i} [req]
  (let [path (get req :path)
        method (get req :method)]
    (match method
      "GET" -> {{:status 200 :id {i}}}
      "POST" -> {{:status 201 :id {i}}}
      "PUT" -> {{:status 200 :id {i}}}
      "DELETE" -> {{:status 204}}
      _ -> {{:status 405}})))

(defn middleware-{i} [handler]
  (fn [req]
    (let [response (handler req)]
      (assoc response :processed true))))
"#,
            i = i
        ));
    }

    // 複雑なパイプライン処理を追加
    for i in 0..30 {
        code.push_str(&format!(
            r#"
(defn pipeline-{i} [data]
  (data
   |> (filter (fn [x] (> x 0)))
   |> (map (fn [x] (* x 2)))
   |> (filter (fn [x] (< x 100)))
   |> (reduce + 0)))
"#,
            i = i
        ));
    }

    // パターンマッチング
    for i in 0..30 {
        code.push_str(&format!(
            r#"
(defn matcher-{i} [value]
  (match value
    0 -> "zero"
    1 | 2 | 3 -> "small"
    n when (< n 10) -> "single-digit"
    n when (< n 100) -> "double-digit"
    [x y] -> "pair"
    [x y ...rest] -> "list"
    {{:name n}} -> "person"
    _ -> "unknown"))
"#,
            i = i
        ));
    }

    code.push_str("\n(export ");
    for i in 0..50 {
        code.push_str(&format!("handler-{i} middleware-{i} "));
    }
    for i in 0..30 {
        code.push_str(&format!("pipeline-{i} matcher-{i} "));
    }
    code.push_str(")\n");

    code
}

/// 構文要素の多様性テスト - 全ての特殊形式を含む
fn diverse_syntax() -> &'static str {
    r#"
;; すべての特殊形式を含むテストコード

;; def, defn, defn-
(def x 42)
(defn public-fn [a b] (+ a b))
(defn- private-fn [x] (* x 2))

;; fn (ラムダ)
(def lambda (fn [x y] (- x y)))
(def variadic (fn [x & rest] (cons x rest)))

;; let
(let [a 1
      b 2
      c (+ a b)]
  (* c 3))

;; if
(if (> 10 5) "yes" "no")

;; do
(do
  (print "step 1")
  (print "step 2")
  (print "step 3")
  42)

;; match
(match [1 2 3]
  [] -> "empty"
  [x] -> "one"
  [x y] -> "two"
  [x y ...rest] -> "many")

;; try
(try (/ 1 0))

;; defer
(defer (print "cleanup"))

;; loop/recur
(loop [i 0 acc 0]
  (if (>= i 10)
    acc
    (recur (+ i 1) (+ acc i))))

;; mac
(mac unless [test & body]
  `(if (not ,test) (do ,@body) nil))

;; flow
(flow |> inc |> (* 2) |> str)

;; module/export/use
(module math-utils)
(export add multiply)
(use std :only [map filter reduce])

;; パイプライン演算子
(1 |> inc |> (* 2))
(1 |>? inc |>? (* 2))
([1 2 3] ||> inc)

;; パターン分解
(defn destructure-test [[x y] {:name n :age a}]
  [x y n a])

;; クオート
'(1 2 3)
`(1 2 ,(+ 1 2))
`(1 2 ,@[3 4 5])

;; データリテラル
[1 2 3 4 5]
{:key "value" :num 42}

;; @deref
@(atom 42)
"#
}

fn bench_lexer_small(c: &mut Criterion) {
    c.bench_function("lexer/small_code", |b| {
        b.iter(|| {
            let mut lexer = qi_lang::lexer::Lexer::new(black_box(small_code()));
            lexer.tokenize().unwrap()
        })
    });
}

fn bench_lexer_medium(c: &mut Criterion) {
    c.bench_function("lexer/medium_code", |b| {
        b.iter(|| {
            let mut lexer = qi_lang::lexer::Lexer::new(black_box(medium_code()));
            lexer.tokenize().unwrap()
        })
    });
}

fn bench_lexer_large(c: &mut Criterion) {
    let code = large_code();
    c.bench_function("lexer/large_code", |b| {
        b.iter(|| {
            let mut lexer = qi_lang::lexer::Lexer::new(black_box(&code));
            lexer.tokenize().unwrap()
        })
    });
}

fn bench_lexer_diverse(c: &mut Criterion) {
    c.bench_function("lexer/diverse_syntax", |b| {
        b.iter(|| {
            let mut lexer = qi_lang::lexer::Lexer::new(black_box(diverse_syntax()));
            lexer.tokenize().unwrap()
        })
    });
}

fn bench_parser_small(c: &mut Criterion) {
    c.bench_function("parser/small_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(small_code())).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_parser_medium(c: &mut Criterion) {
    c.bench_function("parser/medium_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(medium_code())).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_parser_large(c: &mut Criterion) {
    let code = large_code();
    c.bench_function("parser/large_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&code)).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_parser_diverse(c: &mut Criterion) {
    c.bench_function("parser/diverse_syntax", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(diverse_syntax())).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_full_pipeline_small(c: &mut Criterion) {
    c.bench_function("full_pipeline/small_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(small_code())).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_full_pipeline_medium(c: &mut Criterion) {
    c.bench_function("full_pipeline/medium_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(medium_code())).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

fn bench_full_pipeline_large(c: &mut Criterion) {
    let code = large_code();
    c.bench_function("full_pipeline/large_code", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(&code)).unwrap();
            parser.parse_all().unwrap()
        })
    });
}

criterion_group!(
    lexer_benches,
    bench_lexer_small,
    bench_lexer_medium,
    bench_lexer_large,
    bench_lexer_diverse
);

criterion_group!(
    parser_benches,
    bench_parser_small,
    bench_parser_medium,
    bench_parser_large,
    bench_parser_diverse
);

criterion_group!(
    pipeline_benches,
    bench_full_pipeline_small,
    bench_full_pipeline_medium,
    bench_full_pipeline_large
);

criterion_main!(lexer_benches, parser_benches, pipeline_benches);
