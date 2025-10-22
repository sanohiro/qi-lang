//! MySQL統合テスト
//!
//! testcontainersを使用してDockerコンテナを自動起動・削除
//! 実行方法: cargo test --features integration-tests --test integration_mysql

#![cfg(feature = "integration-tests")]

use qi_lang::eval::Evaluator;
use qi_lang::parser::Parser;
use qi_lang::value::Value;
use testcontainers::{clients, GenericImage};

/// MySQLイメージを作成
fn mysql_image() -> GenericImage {
    GenericImage::new("mysql", "8.0")
        .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
            "port: 3306  MySQL Community Server",
        ))
        .with_env_var("MYSQL_ROOT_PASSWORD", "root")
        .with_env_var("MYSQL_DATABASE", "test_db")
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
fn test_mysql_basic_connection() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
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
fn test_mysql_create_table() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
    let mut evaluator = Evaluator::new();

    // 接続
    let conn_code = format!(r#"(def conn (db/connect "{}"))"#, url);
    eval_qi(&mut evaluator, &conn_code).unwrap();

    // テーブル作成
    let create_table = r#"
        (db/exec conn
          "CREATE TABLE users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL
          )" [])
    "#;

    let result = eval_qi(&mut evaluator, create_table);
    assert!(result.is_ok(), "テーブル作成失敗: {:?}", result);
}

#[test]
fn test_mysql_insert_and_query() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
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
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // データ挿入（MySQLは?プレースホルダー）
    let insert = r#"
        (db/exec conn
          "INSERT INTO users (name, email) VALUES (?, ?)"
          ["Alice" "alice@example.com"])
    "#;
    let result = eval_qi(&mut evaluator, insert).unwrap();
    assert_eq!(result, Value::Integer(1.0 as i64), "1行挿入されるべき");

    // データ取得
    let query = r#"
        (db/query conn "SELECT name, email FROM users WHERE name = ?" ["Alice"])
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
fn test_mysql_transaction() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
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
            id INT AUTO_INCREMENT PRIMARY KEY,
            balance INT NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // 初期データ挿入
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO accounts (balance) VALUES (?)" [1000])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO accounts (balance) VALUES (?)" [500])"#,
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
fn test_mysql_rollback() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
    let mut evaluator = Evaluator::new();

    // 接続 & テーブル作成
    eval_qi(
        &mut evaluator,
        &format!(r#"(def conn (db/connect "{}"))"#, url),
    )
    .unwrap();
    eval_qi(&mut evaluator, r#"
        (db/exec conn "CREATE TABLE items (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))" [])
    "#).unwrap();

    // トランザクション開始 → 挿入 → ロールバック
    let tx_code = r#"
        (do
          (def tx (db/begin conn))
          (db/exec tx "INSERT INTO items (name) VALUES (?)" ["test"])
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
fn test_mysql_multiple_inserts() {
    let docker = clients::Cli::default();
    let container = docker.run(mysql_image());
    let port = container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:root@127.0.0.1:{}/test_db", port);
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
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            price INT NOT NULL
          )" [])
    "#,
    )
    .unwrap();

    // 複数データ挿入
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Apple" 100])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Banana" 50])"#,
    )
    .unwrap();
    eval_qi(
        &mut evaluator,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Orange" 150])"#,
    )
    .unwrap();

    // 全件取得
    let result = eval_qi(
        &mut evaluator,
        r#"
        (db/query conn "SELECT name, price FROM products ORDER BY price" [])
    "#,
    )
    .unwrap();

    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 3);
            match (&rows[0], &rows[1], &rows[2]) {
                (Value::Map(r1), Value::Map(r2), Value::Map(r3)) => {
                    assert_eq!(r1.get("name"), Some(&Value::String("Banana".to_string())));
                    assert_eq!(r2.get("name"), Some(&Value::String("Apple".to_string())));
                    assert_eq!(r3.get("name"), Some(&Value::String("Orange".to_string())));
                }
                _ => panic!("期待: Map、実際: {:?}", rows),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}
