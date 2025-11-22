use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{MapKey, Value};

/// データベースエラー
#[derive(Debug, Clone)]
pub struct DbError {
    pub message: String,
}

impl DbError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl From<String> for DbError {
    fn from(s: String) -> Self {
        DbError::new(s)
    }
}

impl From<&str> for DbError {
    fn from(s: &str) -> Self {
        DbError::new(s)
    }
}

pub type DbResult<T> = Result<T, DbError>;

/// マップから値を取得（キーワード、シンボル、文字列のいずれかで）
fn get_map_value<'a>(map: &'a crate::HashMap<MapKey, Value>, key: &str) -> Option<&'a Value> {
    // キーワードで試す（最も一般的）
    if let Ok(keyword_key) = Value::Keyword(crate::intern::intern_keyword(key)).to_map_key() {
        if let Some(v) = map.get(&keyword_key) {
            return Some(v);
        }
    }
    // シンボルで試す
    if let Ok(symbol_key) = Value::Symbol(crate::intern::intern_symbol(key)).to_map_key() {
        if let Some(v) = map.get(&symbol_key) {
            return Some(v);
        }
    }
    // 文字列で試す（後方互換性）
    if let Some(v) = map.get(&MapKey::String(key.to_string())) {
        return Some(v);
    }
    None
}

/// データベース行（カラム名 -> 値のマップ）
pub type Row = crate::HashMap<MapKey, Value>;

/// クエリ結果（行のベクター）
pub type Rows = Vec<Row>;

/// データベース接続オプション
#[derive(Debug, Clone)]
pub struct ConnectionOptions {
    pub timeout_ms: Option<u64>,
    pub read_only: bool,
    pub auto_commit: bool,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000), // 30秒
            read_only: false,
            auto_commit: true,
        }
    }
}

impl ConnectionOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            // timeout - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Integer(ms)) = get_map_value(map, "timeout") {
                options.timeout_ms = Some(*ms as u64);
            }
            // read-only - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Bool(ro)) = get_map_value(map, "read-only") {
                options.read_only = *ro;
            }
            // auto-commit - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Bool(ac)) = get_map_value(map, "auto-commit") {
                options.auto_commit = *ac;
            }
        }

        Ok(options)
    }
}

/// クエリオプション
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub timeout_ms: Option<u64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(5000), // 5秒
            limit: None,
            offset: None,
        }
    }
}

impl QueryOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            // timeout - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Integer(ms)) = get_map_value(map, "timeout") {
                options.timeout_ms = Some(*ms as u64);
            }
            // limit - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Integer(n)) = get_map_value(map, "limit") {
                options.limit = Some(*n);
            }
            // offset - キーワード、シンボル、文字列すべて試す
            if let Some(Value::Integer(n)) = get_map_value(map, "offset") {
                options.offset = Some(*n);
            }
        }

        Ok(options)
    }
}

/// カラム情報
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

/// インデックス情報
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// 外部キー情報
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
}

/// ストアドプロシージャ/ファンクション呼び出しの結果
#[derive(Debug, Clone)]
pub enum CallResult {
    /// 関数の戻り値
    Value(Value),
    /// プロシージャの結果セット
    Rows(Rows),
    /// 複数の結果セット
    Multiple(Vec<Rows>),
}

/// ドライバー情報
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
    pub database_version: String,
}

/// クエリ情報
#[derive(Debug, Clone)]
pub struct QueryInfo {
    pub columns: Vec<ColumnInfo>,
    pub parameter_count: usize,
}

/// トランザクション分離レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl std::str::FromStr for IsolationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read-uncommitted" => Ok(Self::ReadUncommitted),
            "read-committed" => Ok(Self::ReadCommitted),
            "repeatable-read" => Ok(Self::RepeatableRead),
            "serializable" => Ok(Self::Serializable),
            _ => Err(fmt_msg(MsgKey::InvalidIsolationLevel, &[s])),
        }
    }
}

/// トランザクションオプション
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub isolation: IsolationLevel,
    pub timeout_ms: Option<u64>,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation: IsolationLevel::ReadCommitted,
            timeout_ms: Some(10000), // 10秒
        }
    }
}

impl TransactionOptions {
    /// Valueからオプションを構築
    pub fn from_value(opts: &Value) -> DbResult<Self> {
        let mut options = Self::default();

        if let Value::Map(map) = opts {
            if let Some(Value::String(iso)) =
                map.get(&crate::value::MapKey::String("isolation".to_string()))
            {
                options.isolation = iso.parse().map_err(DbError::new)?;
            }
            if let Some(Value::Integer(ms)) =
                map.get(&crate::value::MapKey::String("timeout".to_string()))
            {
                options.timeout_ms = Some(*ms as u64);
            }
        }

        Ok(options)
    }
}
