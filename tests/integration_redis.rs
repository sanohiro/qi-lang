//! Redis統合テスト
//!
//! testcontainersを使用してDockerコンテナを自動起動・削除
//! 実行方法: cargo test --features integration-tests --test integration_redis

#![cfg(feature = "integration-tests")]

use qi_lang::eval::Evaluator;
use qi_lang::parser::Parser;
use qi_lang::value::Value;
use testcontainers::{clients, GenericImage};

/// Redisイメージを作成
fn redis_image() -> GenericImage {
    GenericImage::new("redis", "7")
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
fn test_redis_basic_connection() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続テスト
    let code = format!(r#"(kvs/connect "{}")"#, url);
    let result = eval_qi(&mut evaluator, &code);
    assert!(result.is_ok(), "接続失敗: {:?}", result);

    // 接続IDが返されることを確認
    match result.unwrap() {
        Value::String(s) => assert!(s.starts_with("KvsConnection:")),
        other => panic!("期待: 接続ID文字列、実際: {:?}", other),
    }
}

#[test]
fn test_redis_get_set() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // SET操作
    let set_result = eval_qi(&mut evaluator, r#"(kvs/set conn "test-key" "test-value")"#).unwrap();
    assert_eq!(set_result, Value::String("OK".to_string()));

    // GET操作
    let get_result = eval_qi(&mut evaluator, r#"(kvs/get conn "test-key")"#).unwrap();
    assert_eq!(get_result, Value::String("test-value".to_string()));
}

#[test]
fn test_redis_del() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // SET → DEL → GET
    eval_qi(&mut evaluator, r#"(kvs/set conn "key-to-delete" "value")"#).unwrap();
    let del_result = eval_qi(&mut evaluator, r#"(kvs/del conn "key-to-delete")"#).unwrap();
    assert_eq!(del_result, Value::Integer(1.0 as i64), "1個のキーを削除");

    // 削除後はnilが返る
    let get_result = eval_qi(&mut evaluator, r#"(kvs/get conn "key-to-delete")"#).unwrap();
    assert_eq!(get_result, Value::Nil);
}

#[test]
fn test_redis_exists() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // 存在しないキー
    let exists_result = eval_qi(&mut evaluator, r#"(kvs/exists conn "nonexistent")"#).unwrap();
    assert_eq!(exists_result, Value::Bool(false));

    // SET後は存在する
    eval_qi(&mut evaluator, r#"(kvs/set conn "existing-key" "value")"#).unwrap();
    let exists_result = eval_qi(&mut evaluator, r#"(kvs/exists conn "existing-key")"#).unwrap();
    assert_eq!(exists_result, Value::Bool(true));
}

#[test]
fn test_redis_expire() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // SET → EXPIRE
    eval_qi(&mut evaluator, r#"(kvs/set conn "expiring-key" "value")"#).unwrap();
    let expire_result = eval_qi(&mut evaluator, r#"(kvs/expire conn "expiring-key" 10)"#).unwrap();
    assert_eq!(expire_result, Value::Bool(true));

    // TTL確認（10秒未満であることを確認）
    let ttl_result = eval_qi(&mut evaluator, r#"(kvs/ttl conn "expiring-key")"#).unwrap();
    match ttl_result {
        Value::Integer(ttl) => assert!(ttl > 0 && ttl <= 10, "TTLは0〜10秒の範囲であるべき"),
        other => panic!("期待: Integer、実際: {:?}", other),
    }
}

#[test]
fn test_redis_incr_decr() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // INCR（初期値0から1ずつ増加）
    let incr1 = eval_qi(&mut evaluator, r#"(kvs/incr conn "counter")"#).unwrap();
    assert_eq!(incr1, Value::Integer(1.0 as i64));

    let incr2 = eval_qi(&mut evaluator, r#"(kvs/incr conn "counter")"#).unwrap();
    assert_eq!(incr2, Value::Integer(2.0 as i64));

    // DECR
    let decr1 = eval_qi(&mut evaluator, r#"(kvs/decr conn "counter")"#).unwrap();
    assert_eq!(decr1, Value::Integer(1.0 as i64));
}

#[test]
fn test_redis_mget_mset() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // MSET（複数キー一括設定）
    let mset_result = eval_qi(
        &mut evaluator,
        r#"
        (kvs/mset conn {"key1" "value1" "key2" "value2" "key3" "value3"})
    "#,
    )
    .unwrap();
    assert_eq!(mset_result, Value::String("OK".to_string()));

    // MGET（複数キー一括取得）
    let mget_result = eval_qi(
        &mut evaluator,
        r#"
        (kvs/mget conn ["key1" "key2" "key3"])
    "#,
    )
    .unwrap();

    match mget_result {
        Value::Vector(values) => {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::String("value1".to_string()));
            assert_eq!(values[1], Value::String("value2".to_string()));
            assert_eq!(values[2], Value::String("value3".to_string()));
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_redis_keys() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // 複数のキーを設定
    eval_qi(&mut evaluator, r#"(kvs/set conn "user:1" "Alice")"#).unwrap();
    eval_qi(&mut evaluator, r#"(kvs/set conn "user:2" "Bob")"#).unwrap();
    eval_qi(&mut evaluator, r#"(kvs/set conn "product:1" "Laptop")"#).unwrap();

    // パターンマッチングでキー取得
    let keys_result = eval_qi(&mut evaluator, r#"(kvs/keys conn "user:*")"#).unwrap();

    match keys_result {
        Value::Vector(keys) => {
            assert_eq!(keys.len(), 2);
            // キーの順序は不定なので、両方含まれていることを確認
            let key_strs: Vec<String> = keys
                .iter()
                .filter_map(|k| {
                    if let Value::String(s) = k {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            assert!(key_strs.contains(&"user:1".to_string()));
            assert!(key_strs.contains(&"user:2".to_string()));
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_redis_lpush_rpush_lrange() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // LPUSH（左から追加）
    eval_qi(&mut evaluator, r#"(kvs/lpush conn "mylist" "a")"#).unwrap();
    eval_qi(&mut evaluator, r#"(kvs/lpush conn "mylist" "b")"#).unwrap();

    // RPUSH（右から追加）
    eval_qi(&mut evaluator, r#"(kvs/rpush conn "mylist" "c")"#).unwrap();

    // LRANGE（範囲取得）
    let lrange_result = eval_qi(&mut evaluator, r#"(kvs/lrange conn "mylist" 0 -1)"#).unwrap();

    match lrange_result {
        Value::Vector(items) => {
            assert_eq!(items.len(), 3);
            // LPUSH は左から追加するので: b, a, c の順
            assert_eq!(items[0], Value::String("b".to_string()));
            assert_eq!(items[1], Value::String("a".to_string()));
            assert_eq!(items[2], Value::String("c".to_string()));
        }
        other => panic!("期待: Vector、実際: {:?}", other),
    }
}

#[test]
fn test_redis_hset_hget() {
    let docker = clients::Cli::default();
    let container = docker.run(redis_image());
    let port = container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);
    let mut evaluator = Evaluator::new();

    // 接続
    eval_qi(&mut evaluator, &format!(r#"(def conn (kvs/connect "{}"))"#, url)).unwrap();

    // HSET（ハッシュに値を設定）
    eval_qi(&mut evaluator, r#"(kvs/hset conn "user:1000" "name" "Alice")"#).unwrap();
    eval_qi(&mut evaluator, r#"(kvs/hset conn "user:1000" "age" "30")"#).unwrap();

    // HGET（ハッシュから値を取得）
    let name_result = eval_qi(&mut evaluator, r#"(kvs/hget conn "user:1000" "name")"#).unwrap();
    assert_eq!(name_result, Value::String("Alice".to_string()));

    let age_result = eval_qi(&mut evaluator, r#"(kvs/hget conn "user:1000" "age")"#).unwrap();
    assert_eq!(age_result, Value::String("30".to_string()));

    // HGETALL（ハッシュ全体を取得）
    let hgetall_result = eval_qi(&mut evaluator, r#"(kvs/hgetall conn "user:1000")"#).unwrap();
    match hgetall_result {
        Value::Map(map) => {
            assert_eq!(
                map.get("name"),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(
                map.get("age"),
                Some(&Value::String("30".to_string()))
            );
        }
        other => panic!("期待: Map、実際: {:?}", other),
    }
}
