//! PostgreSQL統合テスト
//!
//! testcontainersを使用してDockerコンテナを自動起動・削除
//! 実行方法: cargo test --features integration-tests --test integration_postgres

#![cfg(feature = "integration-tests")]

use qi_lang::eval::Evaluator;
use qi_lang::parser::Parser;
use qi_lang::value::Value;
use testcontainers::{clients, GenericImage};

/// PostgreSQLイメージを作成
fn postgres_image() -> GenericImage {
    GenericImage::new("postgres", "14")
        .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_HOST_AUTH_METHOD", "trust")
}

/// ヘルパー: Qiコードを評価して結果を返す
fn eval_qi(evaluator: &mut Evaluator, code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code)?;
    let exprs = parser.parse_all()?;

    let mut result = Value::Nil;
    for expr in exprs {
        result = evaluator.eval(&expr)?;
    }
    Ok(result)
}

#[test]
fn test_postgres_basic_connection() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);

    let mut evaluator = Evaluator::new();

    // 接続テスト
    let code = format!(r#"(db/connect "{}")"#, url);
    let result = eval_qi(&mut evaluator, &code);
    assert!(result.is_ok(), "接続失敗: {:?}", result);

    // 接続IDが返されることを確認
    match result.unwrap() {
        Value::String(s) => assert!(s.starts_with("DbConnection:")),
        other => panic!("期待: 接続ID文字列、実際: {:?}", other),
    }
}

#[test]
fn test_postgres_create_table() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続
    let conn_code = format!(r#"(def conn (db/connect "{}"))"#, url);
    eval_qi(&mut evaluator, &conn_code).unwrap();

    // テーブル作成
    let create_table = r#"
        (db/exec conn
          "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL
          )" [])
    "#;

    let result = eval_qi(&mut evaluator, create_table);
    assert!(result.is_ok(), "テーブル作成失敗: {:?}", result);
}

#[test]
fn test_postgres_insert_and_query() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続 & テーブル作成
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn
          "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // データ挿入
    let insert = r#"
        (db/exec conn
          "INSERT INTO users (name, email) VALUES ($1, $2)"
          ["Alice" "alice@example.com"])
    "#;
    let result = eval_qi(&mut evaluator, insert).unwrap();
    assert_eq!(result, Value::Integer(1.0 as i64), "1行挿入されるべき");

    // データ取得
    let query = r#"
        (db/query conn "SELECT name, email FROM users WHERE name = $1" ["Alice"])
    "#;
    let result = eval_qi(&mut evaluator, query).unwrap();

    // 結果検証
    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 1, "1行取得されるべき");
            match &rows[0] {
                Value::Map(row) => {
                    let name = row.get("name");
                    let email = row.get("email");
                    assert_eq!(name, Some(&Value::String("Alice".to_string())));
                    assert_eq!(email, Some(&Value::String("alice@example.com".to_string())));
                }
                other => panic!("期待: Map、実際: {:?}", other),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_postgres_transaction() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続 & テーブル作成
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn
          "CREATE TABLE accounts (
            id SERIAL PRIMARY KEY,
            balance INTEGER NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // 初期データ挿入
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO accounts (balance) VALUES (1000)" [])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO accounts (balance) VALUES (500)" [])"#,
    )
    .unwrap();

    // トランザクション: 送金処理
    let tx_code = r#"
        (do
          (def tx (db/begin conn))
          (db/exec tx "UPDATE accounts SET balance = balance - 100 WHERE id = 1" [])
          (db/exec tx "UPDATE accounts SET balance = balance + 100 WHERE id = 2" [])
          (db/commit tx))
    "#;
    eval_qi(&mut evaluator, tx_code).unwrap();

    // 残高確認
    let result = eval_qi(
        &mut evaluator,
        r#"(db/query conn "SELECT balance FROM accounts ORDER BY id" [])"#,
    )
    .unwrap();
    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 2);
            match (&rows[0], &rows[1]) {
                (Value::Map(row1), Value::Map(row2)) => {
                    let balance1 = row1.get("balance");
                    let balance2 = row2.get("balance");
                    assert_eq!(balance1, Some(&Value::Integer(900.0 as i64)));
                    assert_eq!(balance2, Some(&Value::Integer(600.0 as i64)));
                }
                _ => panic!("期待: Map、実際: {:?}", rows),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_postgres_rollback() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続 & テーブル作成
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn "CREATE TABLE items (id SERIAL PRIMARY KEY, name TEXT)" [])
    "#,
    )
    .unwrap();

    // トランザクション開始 → 挿入 → ロールバック
    let tx_code = r#"
        (do
          (def tx (db/begin conn))
          (db/exec tx "INSERT INTO items (name) VALUES ($1)" ["test"])
          (db/rollback tx))
    "#;
    eval_qi(&mut evaluator, tx_code).unwrap();

    // データが挿入されていないことを確認
    let result = eval_qi(
        &mut evaluator,
        r#"(db/query conn "SELECT * FROM items" [])"#,
    )
    .unwrap();
    match result {
        Value::Vector(rows) => assert_eq!(rows.len(), 0, "ロールバック後は0行であるべき"),
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_postgres_parameterized_query() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続 & テーブル作成
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn
          "CREATE TABLE products (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            price INTEGER NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // 複数データ挿入
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES ($1, $2)" ["Apple" 100])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES ($1, $2)" ["Banana" 50])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES ($1, $2)" ["Orange" 150])"#,
    )
    .unwrap();

    // パラメータ化クエリで検索
    let result = eval_qi(
        &mut evaluator,
        r#"
        (db/query conn "SELECT name FROM products WHERE price > $1 ORDER BY price" [80])
    "#,
    )
    .unwrap();

    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 2);
            match (&rows[0], &rows[1]) {
                (Value::Map(r1), Value::Map(r2)) => {
                    assert_eq!(r1.get("name"), Some(&Value::String("Apple".to_string())));
                    assert_eq!(r2.get("name"), Some(&Value::String("Orange".to_string())));
                }
                _ => panic!("期待: Map、実際: {:?}", rows),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_postgres_metadata() {
    let docker = clients::Cli::default();
    let container = docker.run(postgres_image());
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgresql://postgres@127.0.0.1:{}/postgres", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();

    // テーブル作成
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn "CREATE TABLE products (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            price INTEGER DEFAULT 0,
            category TEXT
        )" [])
        "#,
    )
    .unwrap();

    // インデックス作成
    eval_qi(
        &mut evaluator,
        r#"
        (db/exec conn "CREATE INDEX idx_category ON products(category)" [])
        "#,
    )
    .unwrap();

    // db/tables - テーブル一覧
    let tables = eval_qi(&mut evaluator, r#"(db/tables conn)"#).unwrap();
    match tables {
        Value::Vector(vec) => {
            assert!(vec.len() > 0);
            let has_products = vec
                .iter()
                .any(|v| matches!(v, Value::String(s) if s == "products"));
            assert!(has_products, "productsテーブルが見つかりません");
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }

    // db/columns - カラム情報
    let columns = eval_qi(&mut evaluator, r#"(db/columns conn "products")"#).unwrap();
    match columns {
        Value::Vector(vec) => {
            assert_eq!(vec.len(), 4, "カラム数が4個であるべき");
            // カラム情報を確認（詳細なチェックはせず、情報が取得できることを確認）
            for col in &vec {
                if let Value::Map(m) = col {
                    assert!(m.contains_key("name"), "カラムにnameフィールドが必要");
                    assert!(m.contains_key("type"), "カラムにtypeフィールドが必要");
                    assert!(
                        m.contains_key("nullable"),
                        "カラムにnullableフィールドが必要"
                    );
                    assert!(
                        m.contains_key("primary_key"),
                        "カラムにprimary_keyフィールドが必要"
                    );
                }
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }

    // db/indexes - インデックス情報
    let indexes = eval_qi(&mut evaluator, r#"(db/indexes conn "products")"#).unwrap();
    match indexes {
        Value::Vector(vec) => {
            assert!(vec.len() >= 1, "少なくとも1つのインデックスがあるべき");
            // インデックス名を確認
            let has_idx_category = vec.iter().any(|v| {
                if let Value::Map(idx) = v {
                    matches!(idx.get("name"), Some(Value::String(s)) if s.contains("idx_category"))
                } else {
                    false
                }
            });
            assert!(has_idx_category, "idx_categoryインデックスが見つかりません");
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }

    // db/driver-info - ドライバ情報
    let driver_info = eval_qi(&mut evaluator, r#"(db/driver-info conn)"#).unwrap();
    match driver_info {
        Value::Map(info) => {
            assert_eq!(
                info.get("name"),
                Some(&Value::String("PostgreSQL".to_string())),
                "ドライバ名"
            );
            assert!(info.contains_key("version"), "versionフィールドが必要");
            assert!(
                info.contains_key("database_version"),
                "database_versionフィールドが必要"
            );
        }
        other => panic!("期待: Map、実際: {:?}", other),
    }
}
