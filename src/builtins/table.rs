//! テーブル処理関数
//!
//! CSV、JSON、データベース結果などの表形式データを扱うための関数群。
//! awk/SQL風のデータ操作を提供する。

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use crate::{check_args, new_hashmap};

/// テーブルデータの形式
#[derive(Debug, Clone)]
enum TableFormat {
    /// [{"name": "Alice", "age": 30}, ...]
    MapList,
    /// [["name", "age"], ["Alice", 30], ...]
    ArrayWithHeader,
    /// [[100, 200], [300, 400]]
    ArrayNoHeader,
}

/// テーブルデータの表現
#[derive(Debug, Clone)]
struct Table {
    format: TableFormat,
    headers: Option<Vec<String>>,
    rows: Vec<Vec<Value>>,
}

impl Table {
    /// Valueからテーブルを抽出
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::List(items) | Value::Vector(items) => {
                if items.is_empty() {
                    return Ok(Table {
                        format: TableFormat::ArrayNoHeader,
                        headers: None,
                        rows: Vec::new(),
                    });
                }

                // im::VectorをVec<Value>に変換
                let items_vec: Vec<Value> = items.iter().cloned().collect();

                // 最初の要素で形式を判別
                match &items_vec[0] {
                    Value::Map(_) => Self::from_map_list(&items_vec),
                    Value::List(_) | Value::Vector(_) => Self::from_array_list(&items_vec),
                    _ => Err(fmt_msg(
                        MsgKey::TableInvalidFormat,
                        &["list of maps or list of lists"],
                    )),
                }
            }
            _ => Err(fmt_msg(MsgKey::TableInvalidFormat, &["list or vector"])),
        }
    }

    /// MapList形式からテーブルを構築
    fn from_map_list(items: &[Value]) -> Result<Self, String> {
        let mut rows = Vec::new();

        // 全ての行のキーの和集合を取得（異質データ対応）
        let mut all_keys = std::collections::HashSet::new();
        for item in items {
            if let Value::Map(map) = item {
                for key in map.keys() {
                    all_keys.insert(key.clone());
                }
            } else {
                return Err(fmt_msg(
                    MsgKey::TableInvalidFormat,
                    &["all elements must be maps"],
                ));
            }
        }

        // キーをソート（順序を安定させる）
        let mut pairs: Vec<(crate::value::MapKey, String)> = all_keys
            .into_iter()
            .map(|k| (k.clone(), k.to_string()))
            .collect();
        pairs.sort_by(|a, b| a.1.cmp(&b.1));
        let (header_keys, header_strings): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();

        // 各行を抽出（存在しないキーはnilで埋める）
        for item in items {
            if let Value::Map(map) = item {
                let mut row = Vec::new();
                for key in &header_keys {
                    row.push(map.get(key).cloned().unwrap_or(Value::Nil));
                }
                rows.push(row);
            } else {
                return Err(fmt_msg(
                    MsgKey::TableInvalidFormat,
                    &["all elements must be maps"],
                ));
            }
        }

        Ok(Table {
            format: TableFormat::MapList,
            headers: Some(header_strings),
            rows,
        })
    }

    /// ArrayList形式からテーブルを構築
    fn from_array_list(items: &[Value]) -> Result<Self, String> {
        let mut rows = Vec::new();
        let headers: Option<Vec<String>>;

        // 最初の行が文字列のみかチェック（ヘッダー行の可能性）
        let first_row = match &items[0] {
            Value::List(r) | Value::Vector(r) => r,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TableInvalidFormat,
                    &["all elements must be lists"],
                ))
            }
        };

        let has_header = first_row.iter().all(|v| matches!(v, Value::String(_)));

        if has_header && items.len() > 1 {
            // ヘッダー行あり
            headers = Some(
                first_row
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    })
                    .collect(),
            );

            // データ行を抽出
            for item in &items[1..] {
                match item {
                    Value::List(r) | Value::Vector(r) => rows.push(r.iter().cloned().collect()),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TableInvalidFormat,
                            &["all elements must be lists"],
                        ))
                    }
                }
            }

            Ok(Table {
                format: TableFormat::ArrayWithHeader,
                headers,
                rows,
            })
        } else {
            // ヘッダー行なし（全行がデータ）
            for item in items {
                match item {
                    Value::List(r) | Value::Vector(r) => rows.push(r.iter().cloned().collect()),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TableInvalidFormat,
                            &["all elements must be lists"],
                        ))
                    }
                }
            }

            Ok(Table {
                format: TableFormat::ArrayNoHeader,
                headers: None,
                rows,
            })
        }
    }

    /// テーブルをValueに変換
    fn to_value(&self) -> Value {
        match self.format {
            TableFormat::MapList => {
                // MapList形式は常にheadersを持つ（from_map_list参照）
                #[allow(clippy::expect_used)]
                let headers = self
                    .headers
                    .as_ref()
                    .expect("TableFormat::MapList requires headers");
                let maps: Vec<Value> = self
                    .rows
                    .iter()
                    .map(|row| {
                        let mut map = new_hashmap();
                        for (i, header) in headers.iter().enumerate() {
                            if i < row.len() {
                                map.insert(
                                    crate::value::MapKey::String(header.clone()),
                                    row[i].clone(),
                                );
                            }
                        }
                        Value::Map(map)
                    })
                    .collect();
                Value::List(maps.into())
            }
            TableFormat::ArrayWithHeader => {
                let mut result = Vec::new();
                if let Some(headers) = &self.headers {
                    result.push(Value::List(
                        headers.iter().map(|h| Value::String(h.clone())).collect(),
                    ));
                }
                for row in &self.rows {
                    result.push(Value::List(row.clone().into()));
                }
                Value::List(result.into())
            }
            TableFormat::ArrayNoHeader => {
                let result: Vec<Value> = self
                    .rows
                    .iter()
                    .map(|row| Value::List(row.clone().into()))
                    .collect();
                Value::List(result.into())
            }
        }
    }

    /// カラムインデックスを解決
    fn resolve_column(&self, selector: &Value) -> Result<usize, String> {
        match selector {
            Value::String(name) => {
                // 名前でアクセス（String）
                if let Some(headers) = &self.headers {
                    headers
                        .iter()
                        .position(|h| h == name)
                        .ok_or_else(|| fmt_msg(MsgKey::TableColumnNotFound, &[name]))
                } else {
                    Err(fmt_msg(MsgKey::TableNoHeaders, &[]))
                }
            }
            Value::Keyword(name) => {
                // 名前でアクセス（Keyword）
                if let Some(headers) = &self.headers {
                    let search_key = format!(":{}", name);
                    headers
                        .iter()
                        .position(|h| h == &search_key)
                        .ok_or_else(|| fmt_msg(MsgKey::TableColumnNotFound, &[&search_key]))
                } else {
                    Err(fmt_msg(MsgKey::TableNoHeaders, &[]))
                }
            }
            Value::Integer(idx) => {
                // インデックスでアクセス
                let col_count = self.rows.first().map(|r| r.len()).unwrap_or(0);
                let resolved_idx = if *idx < 0 {
                    // 負数: 末尾から
                    (col_count as i64 + idx) as usize
                } else {
                    *idx as usize
                };

                if resolved_idx < col_count {
                    Ok(resolved_idx)
                } else {
                    Err(fmt_msg(
                        MsgKey::TableColumnIndexOutOfRange,
                        &[&idx.to_string()],
                    ))
                }
            }
            _ => Err(fmt_msg(
                MsgKey::TableColumnSelectorInvalid,
                &["string, keyword, or integer"],
            )),
        }
    }
}

// ========================================
// 関数実装
// ========================================

/// table/where - 述語関数で行をフィルタリング
///
/// 引数: (table, predicate-fn)
/// 例: (table/where data (fn [row] (> (get row "age") 25)))
///
/// 注: この関数はEvaluatorが必要なため、NativeEvalFnとして実装されています。
pub fn native_table_where(args: &[Value], eval: &Evaluator) -> Result<Value, String> {
    check_args!(args, 2, "table/where");

    let table = Table::from_value(&args[0])?;
    let predicate = &args[1];

    let mut filtered_rows = Vec::new();
    for row in &table.rows {
        // 各行をMapまたはVectorに変換（元の形式に応じて）
        let row_value = match &table.format {
            TableFormat::MapList => {
                // MapList形式はMapとして渡す（キーは":key"形式）
                // MapList形式は常にheadersを持つ（from_map_list参照）
                #[allow(clippy::expect_used)]
                let headers = table
                    .headers
                    .as_ref()
                    .expect("TableFormat::MapList requires headers");
                let mut map = new_hashmap();
                for (i, header) in headers.iter().enumerate() {
                    map.insert(
                        crate::value::MapKey::String(header.clone()),
                        row.get(i).cloned().unwrap_or(Value::Nil),
                    );
                }
                Value::Map(map)
            }
            TableFormat::ArrayWithHeader | TableFormat::ArrayNoHeader => {
                // 配列形式（ヘッダーの有無に関わらず）はVectorとして渡す
                // ユーザーはnthでインデックスアクセスできる
                Value::Vector(row.clone().into())
            }
        };

        // 述語関数を呼び出し
        let result = eval.apply_function(predicate, &[row_value])?;

        if result.is_truthy() {
            filtered_rows.push(row.clone());
        }
    }

    let new_table = Table {
        format: table.format,
        headers: table.headers,
        rows: filtered_rows,
    };

    Ok(new_table.to_value())
}

/// table/select - 指定した列のみ抽出
///
/// 引数: (table, column-selectors)
/// 例: (table/select data ["name" "age"])
///     (table/select data [0 2])  ;; インデックス指定
pub fn native_table_select(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "table/select");

    let table = Table::from_value(&args[0])?;
    let selectors = match &args[1] {
        Value::List(s) | Value::Vector(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TableSelectNeedsList,
                &["list or vector of column selectors"],
            ))
        }
    };

    // カラムインデックスを解決
    let col_indices: Result<Vec<usize>, String> = selectors
        .iter()
        .map(|sel| table.resolve_column(sel))
        .collect();
    let col_indices = col_indices?;

    // 新しいヘッダーと行を作成
    let new_headers = table.headers.as_ref().map(|headers| {
        col_indices
            .iter()
            .map(|&i| headers.get(i).cloned().unwrap_or_default())
            .collect()
    });

    let new_rows: Vec<Vec<Value>> = table
        .rows
        .iter()
        .map(|row| {
            col_indices
                .iter()
                .map(|&i| row.get(i).cloned().unwrap_or(Value::Nil))
                .collect()
        })
        .collect();

    let new_table = Table {
        format: table.format.clone(),
        headers: new_headers,
        rows: new_rows,
    };

    Ok(new_table.to_value())
}

/// table/order-by - 指定列でソート
///
/// 引数: (table, column-selector [:asc/:desc])
/// 例: (table/order-by data "age")
///     (table/order-by data 1 :desc)
pub fn native_table_order_by(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(fmt_msg(MsgKey::NeedNArgs, &["table/order-by", "2 or 3"]));
    }

    let mut table = Table::from_value(&args[0])?;
    let col_idx = table.resolve_column(&args[1])?;

    // ソート順序
    let descending = if args.len() == 3 {
        match &args[2] {
            Value::Keyword(k) if &**k == "desc" => true,
            Value::Keyword(k) if &**k == "asc" => false,
            _ => {
                return Err(fmt_msg(
                    MsgKey::TableOrderByInvalidOrder,
                    &[":asc or :desc"],
                ))
            }
        }
    } else {
        false // デフォルトは昇順
    };

    // ソート
    table.rows.sort_by(|a, b| {
        let val_a = a.get(col_idx).unwrap_or(&Value::Nil);
        let val_b = b.get(col_idx).unwrap_or(&Value::Nil);

        let cmp = compare_values(val_a, val_b);
        if descending {
            cmp.reverse()
        } else {
            cmp
        }
    });

    Ok(table.to_value())
}

/// table/take - 先頭N行を取得
///
/// 引数: (table, n)
/// 例: (table/take data 10)
pub fn native_table_take(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "table/take");

    let mut table = Table::from_value(&args[0])?;
    let n = match &args[1] {
        Value::Integer(i) if *i >= 0 => *i as usize,
        Value::Integer(i) => return Err(fmt_msg(MsgKey::TableTakeNegative, &[&i.to_string()])),
        _ => return Err(fmt_msg(MsgKey::TableTakeNotInteger, &[])),
    };

    table.rows.truncate(n);
    Ok(table.to_value())
}

/// table/drop - 先頭N行をスキップ
///
/// 引数: (table, n)
/// 例: (table/drop data 5)
pub fn native_table_drop(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "table/drop");

    let mut table = Table::from_value(&args[0])?;
    let n = match &args[1] {
        Value::Integer(i) if *i >= 0 => *i as usize,
        Value::Integer(i) => return Err(fmt_msg(MsgKey::TableDropNegative, &[&i.to_string()])),
        _ => return Err(fmt_msg(MsgKey::TableDropNotInteger, &[])),
    };

    if n < table.rows.len() {
        table.rows = table.rows.split_off(n);
    } else {
        table.rows.clear();
    }

    Ok(table.to_value())
}

// ========================================
// ヘルパー関数
// ========================================

/// 値の比較（ソート用）
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => {
            if x < y {
                Ordering::Less
            } else if x > y {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        (Value::Integer(x), Value::Float(y)) => {
            let x = *x as f64;
            if x < *y {
                Ordering::Less
            } else if x > *y {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        (Value::Float(x), Value::Integer(y)) => {
            let y = *y as f64;
            if x < &y {
                Ordering::Less
            } else if x > &y {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        (Value::String(x), Value::String(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (Value::Nil, Value::Nil) => Ordering::Equal,
        (Value::Nil, _) => Ordering::Less,
        (_, Value::Nil) => Ordering::Greater,
        _ => Ordering::Equal, // 比較不能な型はそのまま
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// @qi-doc:category table
/// @qi-doc:functions select, order-by, take, drop, where
/// @qi-doc:note テーブル処理（awk/SQL風）
pub const FUNCTIONS: super::NativeFunctions = &[
    ("table/select", native_table_select),
    ("table/order-by", native_table_order_by),
    ("table/take", native_table_take),
    ("table/drop", native_table_drop),
];

/// Evaluator必要な関数
pub const EVAL_FUNCTIONS: super::NativeEvalFunctions = &[("table/where", native_table_where)];
