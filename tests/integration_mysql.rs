//! MySQL統合テスト
//!
//! testcontainersを使用してDockerコンテナを自動起動・削除
//! 実行方法: cargo test --features integration-tests --test integration_mysql

#![cfg(feature = "integration-tests")]

use qi_lang::env::Env;
use qi_lang::eval::eval;
use qi_lang::parser::parse;
use qi_lang::value::Value;
use std::sync::{Arc, RwLock};
use testcontainers::{clients::Cli, RunnableImage};
use testcontainers_modules::mysql::Mysql;

/// MySQLコンテナをセットアップして接続URLを返す
fn setup_mysql() -> (Cli, testcontainers::ContainerAsync<Mysql>, String) {
    let docker = Cli::default();

    // MySQLイメージ（標準）
    let mysql_image = Mysql::default()
        .with_db_name("test_db")
        .with_user("test_user")
        .with_password("test_password")
        .with_root_password("root_password");

    let container = docker.run(mysql_image);

    // ポート取得（バッティング回避のため動的割り当て）
    let port = container.get_host_port_ipv4(3306);
    let connection_url = format!("mysql://test_user:test_password@127.0.0.1:{}/test_db", port);

    (docker, container, connection_url)
}

/// ヘルパー: Qiコードを評価して結果を返す
fn eval_qi(env: &Arc<RwLock<Env>>, code: &str) -> Result<Value, String> {
    let expr = parse(code)?;
    eval(&expr, env)
}

#[test]
fn test_mysql_basic_connection() {
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続テスト
    let code = format!(r#"(db/connect "{}")"#, url);
    let result = eval_qi(&env, &code);
    assert!(result.is_ok(), "接続失敗: {:?}", result);

    // 接続IDが返されることを確認
    match result.unwrap() {
        Value::String(s) => assert!(s.starts_with("DbConnection:")),
        other => panic!("期待: 接続ID文字列、実際: {:?}", other),
    }
}

#[test]
fn test_mysql_create_table() {
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続
    let conn_code = format!(r#"(def conn (db/connect "{}"))"#, url);
    eval_qi(&env, &conn_code).unwrap();

    // テーブル作成
    let create_table = r#"
        (db/exec conn
          "CREATE TABLE users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL
          )" [])
    "#;

    let result = eval_qi(&env, create_table);
    assert!(result.is_ok(), "テーブル作成失敗: {:?}", result);
}

#[test]
fn test_mysql_insert_and_query() {
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続 & テーブル作成
    eval_qi(&env, &format!(r#"(def conn (db/connect "{}"))"#, url)).unwrap();
    eval_qi(
        &env,
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
    let result = eval_qi(&env, insert).unwrap();
    assert_eq!(result, Value::Number(1.0), "1行挿入されるべき");

    // データ取得
    let query = r#"
        (db/query conn "SELECT name, email FROM users WHERE name = ?" ["Alice"])
    "#;
    let result = eval_qi(&env, query).unwrap();

    // 結果検証
    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 1, "1行取得されるべき");
            match &rows[0] {
                Value::Map(row) => {
                    let name = row.get(&Value::String("name".to_string()));
                    let email = row.get(&Value::String("email".to_string()));
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
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続 & テーブル作成
    eval_qi(&env, &format!(r#"(def conn (db/connect "{}"))"#, url)).unwrap();
    eval_qi(
        &env,
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
        &env,
        r#"(db/exec conn "INSERT INTO accounts (balance) VALUES (?)" [1000])"#,
    )
    .unwrap();
    eval_qi(
        &env,
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
    eval_qi(&env, tx_code).unwrap();

    // 残高確認
    let result = eval_qi(
        &env,
        r#"(db/query conn "SELECT balance FROM accounts ORDER BY id" [])"#,
    )
    .unwrap();
    match result {
        Value::Vector(rows) => {
            assert_eq!(rows.len(), 2);
            match (&rows[0], &rows[1]) {
                (Value::Map(row1), Value::Map(row2)) => {
                    let balance1 = row1.get(&Value::String("balance".to_string()));
                    let balance2 = row2.get(&Value::String("balance".to_string()));
                    assert_eq!(balance1, Some(&Value::Number(900.0)));
                    assert_eq!(balance2, Some(&Value::Number(600.0)));
                }
                _ => panic!("期待: Map、実際: {:?}", rows),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_mysql_rollback() {
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続 & テーブル作成
    eval_qi(&env, &format!(r#"(def conn (db/connect "{}"))"#, url)).unwrap();
    eval_qi(&env, r#"
        (db/exec conn "CREATE TABLE items (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))" [])
    "#).unwrap();

    // トランザクション開始 → 挿入 → ロールバック
    let tx_code = r#"
        (do
          (def tx (db/begin conn))
          (db/exec tx "INSERT INTO items (name) VALUES (?)" ["test"])
          (db/rollback tx))
    "#;
    eval_qi(&env, tx_code).unwrap();

    // データが挿入されていないことを確認
    let result = eval_qi(&env, r#"(db/query conn "SELECT * FROM items" [])"#).unwrap();
    match result {
        Value::Vector(rows) => assert_eq!(rows.len(), 0, "ロールバック後は0行であるべき"),
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_mysql_multiple_inserts() {
    let (_docker, _container, url) = setup_mysql();
    let env = Arc::new(RwLock::new(Env::new()));

    // 接続 & テーブル作成
    eval_qi(&env, &format!(r#"(def conn (db/connect "{}"))"#, url)).unwrap();
    eval_qi(
        &env,
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
        &env,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Apple" 100])"#,
    )
    .unwrap();
    eval_qi(
        &env,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Banana" 50])"#,
    )
    .unwrap();
    eval_qi(
        &env,
        r#"(db/exec conn "INSERT INTO products (name, price) VALUES (?, ?)" ["Orange" 150])"#,
    )
    .unwrap();

    // 全件取得
    let result = eval_qi(
        &env,
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
                    assert_eq!(
                        r1.get(&Value::String("name".to_string())),
                        Some(&Value::String("Banana".to_string()))
                    );
                    assert_eq!(
                        r2.get(&Value::String("name".to_string())),
                        Some(&Value::String("Apple".to_string()))
                    );
                    assert_eq!(
                        r3.get(&Value::String("name".to_string())),
                        Some(&Value::String("Orange".to_string()))
                    );
                }
                _ => panic!("期待: Map、実際: {:?}", rows),
            }
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}
