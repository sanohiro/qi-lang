use crate::constants::keywords::ERROR_KEY;
use crate::lexer::Span;
use crossbeam_channel::{Receiver, Sender};
use im::Vector;
use parking_lot::RwLock;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// f-stringの部品（文字列またはコード）
#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String), // 通常の文字列部分
    Code(String), // {expr} 内のコード
}

/// Qi言語の値を表現する型
#[derive(Debug, Clone)]
pub enum Value {
    /// nil値
    Nil,
    /// bool値
    Bool(bool),
    /// 整数
    Integer(i64),
    /// 浮動小数点数
    Float(f64),
    /// 文字列
    String(String),
    /// シンボル（インターン化でメモリ削減）
    Symbol(Arc<str>),
    /// キーワード（インターン化でメモリ削減・比較高速化）
    Keyword(Arc<str>),
    /// リスト（cons cell）
    List(Vector<Value>),
    /// ベクタ
    Vector(Vector<Value>),
    /// マップ
    Map(crate::HashMap<String, Value>),
    /// 関数（クロージャ）
    Function(Arc<Function>),
    /// ネイティブ関数（Rustで実装された関数）
    NativeFunc(NativeFunc),
    /// マクロ
    Macro(Arc<Macro>),
    /// アトム（可変な参照）
    Atom(Arc<RwLock<Value>>),
    /// チャネル（go/chan並行処理用）
    Channel(Arc<Channel>),
    /// スコープ（Structured Concurrency用）
    Scope(Arc<Scope>),
    /// ストリーム（遅延評価）
    Stream(Arc<RwLock<Stream>>),
    /// ユニーク変数（マクロの衛生性）
    Uvar(u64),
}

/// チャネル（送信・受信両方可能）
///
/// goroutine風の並行処理で使用する通信チャネル。
/// `crossbeam_channel`をラップしており、複数スレッド間で安全に共有できます。
///
/// # 特徴
/// - MPMC（Multiple Producer, Multiple Consumer）
/// - スレッドセーフ
/// - `Arc`でラップされているため、複数のスレッドで共有可能
#[derive(Debug, Clone)]
pub struct Channel {
    pub sender: Sender<Value>,
    pub receiver: Receiver<Value>,
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// 実際の等価性比較はValueのPartialEqでArc::ptr_eqを使用
impl PartialEq for Channel {
    fn eq(&self, _other: &Self) -> bool {
        // チャネルは構造的な等価性比較が困難なため、常にfalse
        false
    }
}

/// スコープ（Structured Concurrency用）
#[derive(Debug, Clone)]
pub struct Scope {
    pub cancelled: Arc<RwLock<bool>>,
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// 実際の等価性比較はValueのPartialEqでArc::ptr_eqを使用
impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cancelled, &other.cancelled)
    }
}

/// ストリーム（遅延評価）
pub struct Stream {
    pub next_fn: Box<dyn Fn() -> Option<Value> + Send + Sync>,
}

impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stream {{ next_fn: <closure> }}")
    }
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// 実際の等価性比較はValueのPartialEqでArc::ptr_eqを使用
impl PartialEq for Stream {
    fn eq(&self, _other: &Self) -> bool {
        // ストリームは関数を持つため構造的な等価性比較が困難
        false
    }
}

impl Value {
    /// 真偽値判定（nilとfalse以外はすべてtruthy）
    #[inline]
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    /// 値をマップのキー文字列に変換
    ///
    /// 異なる型のキーが衝突しないよう、型プレフィックスをつける：
    /// - Keyword: ":name"
    /// - String: "\"text\""
    /// - Symbol: "'symbol"
    /// - Integer: "123"
    /// - Nil: "nil"
    /// - Bool: "true" or "false"
    pub fn to_map_key(&self) -> Result<String, String> {
        use crate::i18n::{fmt_msg, msg, MsgKey};
        match self {
            Value::Keyword(k) => Ok(format!(":{}", k)),
            Value::String(s) => Ok(format!("\"{}\"", s)),
            Value::Symbol(s) => Ok(format!("'{}", s)),
            Value::Integer(n) => Ok(n.to_string()),
            Value::Nil => Ok("nil".to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Float(_) => Err(msg(MsgKey::FloatKeyNotAllowed).to_string()),
            _ => Err(fmt_msg(MsgKey::InvalidMapKey, &[self.type_name()])),
        }
    }

    /// 型名を取得（エラーメッセージ用）
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::Keyword(_) => "keyword",
            Value::List(_) => "list",
            Value::Vector(_) => "vector",
            Value::Map(_) => "map",
            Value::Function(_) => "function",
            Value::NativeFunc(_) => "function",
            Value::Macro(_) => "macro",
            Value::Atom(_) => "atom",
            Value::Channel(_) => "channel",
            Value::Scope(_) => "scope",
            Value::Stream(_) => "stream",
            Value::Uvar(_) => "uvar",
        }
    }

    /// List/Vectorを統一的に扱うヘルパー
    ///
    /// ListまたはVectorの内部データ（im::Vector）への参照を返す
    /// どちらでもない場合はNoneを返す
    pub fn as_seq(&self) -> Option<&im::Vector<Value>> {
        match self {
            Value::List(v) | Value::Vector(v) => Some(v),
            _ => None,
        }
    }

    /// List/Vectorをイテレータとして扱うヘルパー
    ///
    /// ListまたはVectorのイテレータを返す
    /// どちらでもない場合はNoneを返す
    ///
    /// 使用例:
    /// ```ignore
    /// if let Some(iter) = value.as_sequence_iter() {
    ///     for item in iter {
    ///         // itemを処理
    ///     }
    /// }
    /// ```
    pub fn as_sequence_iter(&self) -> Option<impl Iterator<Item = &Value>> {
        match self {
            Value::List(v) | Value::Vector(v) => Some(v.iter()),
            _ => None,
        }
    }

    /// エラー値を生成（{:error "message"}）
    ///
    /// # 例
    /// ```ignore
    /// return Ok(Value::error("file not found"));
    /// ```
    pub fn error(message: impl Into<String>) -> Value {
        let mut map = crate::new_hashmap();
        map.insert(ERROR_KEY.to_string(), Value::String(message.into()));
        Value::Map(map)
    }

    /// 詳細情報付きエラー値を生成（{:error {:type "network" :message "..."}}）
    ///
    /// # 例
    /// ```ignore
    /// return Ok(Value::error_with_details(im::hashmap!{
    ///     "type".to_string() => Value::String("network".to_string()),
    ///     "code".to_string() => Value::Integer(404),
    /// }));
    /// ```
    pub fn error_with_details(details: crate::HashMap<String, Value>) -> Value {
        let mut map = crate::new_hashmap();
        map.insert(ERROR_KEY.to_string(), Value::Map(details));
        Value::Map(map)
    }

    /// 値がエラーかチェック（{:error ...}形式）
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Map(m) if m.contains_key(ERROR_KEY))
    }
}

/// ValueのPartialEq実装（関数やマクロなどはポインタ比較）
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Keyword(a), Value::Keyword(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            // ListとVectorは内容が同じなら等しい（Lisp系言語の一般的な仕様）
            (Value::List(a), Value::Vector(b)) | (Value::Vector(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::NativeFunc(a), Value::NativeFunc(b)) => a == b,
            (Value::Uvar(a), Value::Uvar(b)) => a == b,
            // 関数、マクロ、アトム、チャネル、スコープ、ストリームはポインタ比較
            (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b),
            (Value::Macro(a), Value::Macro(b)) => Arc::ptr_eq(a, b),
            (Value::Atom(a), Value::Atom(b)) => Arc::ptr_eq(a, b),
            (Value::Channel(a), Value::Channel(b)) => Arc::ptr_eq(a, b),
            (Value::Scope(a), Value::Scope(b)) => Arc::ptr_eq(a, b),
            (Value::Stream(a), Value::Stream(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Hashトレイト実装（集合演算の高速化）
///
/// Float, Function, NativeFunc, Macro, Atom, Channel, Scope, Stream, Uvarは
/// ハッシュ化できないため、これらの値を含むコレクションで集合演算を行うと
/// エラーになります。
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // discriminant（列挙子の識別子）をハッシュ化して型の違いを区別
        std::mem::discriminant(self).hash(state);

        match self {
            Value::Nil => {}
            Value::Bool(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::String(s) => s.hash(state),
            Value::Symbol(s) => s.hash(state),
            Value::Keyword(k) => k.hash(state),
            Value::List(items) | Value::Vector(items) => {
                // ListとVectorは同じ内容なら同じハッシュ値を返す
                items.hash(state);
            }
            Value::Map(m) => {
                // MapはHashMapなので、キーと値のペアをソートしてハッシュ化
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash(state);
                }
            }
            // ハッシュ化できない型
            Value::Float(_)
            | Value::Function(_)
            | Value::NativeFunc(_)
            | Value::Macro(_)
            | Value::Atom(_)
            | Value::Channel(_)
            | Value::Scope(_)
            | Value::Stream(_)
            | Value::Uvar(_) => {
                // panicではなく、固定値をハッシュ化（呼び出し元でチェックする）
                // 集合演算では事前に型チェックするため、ここには到達しない想定
                0u8.hash(state);
            }
        }
    }
}

/// Eqトレイト実装（PartialEqに加えて完全な等価性を保証）
///
/// Valueは反射性・対称性・推移性を満たすため、Eqを実装できます。
/// ただし、Floatを含む値は厳密にはEqではありませんが（NaN != NaN）、
/// Qi言語ではFloatの==は通常の浮動小数点比較として扱います。
impl Eq for Value {}

/// 関数の定義
#[derive(Debug, Clone)]
pub struct Function {
    /// 関数のパラメータパターン（Pattern型に統一）
    pub params: Vec<Pattern>,
    pub body: Arc<Expr>,
    pub env: Arc<RwLock<Env>>,
    pub is_variadic: bool, // &argsに対応
    /// 特殊処理が必要な関数フラグ（complement, juxt, tap>）
    /// 通常関数ではfalse、環境ルックアップをスキップして高速化
    pub has_special_processing: bool,
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// 実際の等価性比較はValueのPartialEqでArc::ptr_eqを使用
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        // 関数の等価性はパラメータ、ボディ、可変長フラグで判定
        // 環境は比較しない（クロージャの等価性は意味論的に困難）
        self.params == other.params
            && self.body == other.body
            && self.is_variadic == other.is_variadic
            && self.has_special_processing == other.has_special_processing
    }
}

/// マクロの定義
#[derive(Debug, Clone)]
pub struct Macro {
    pub name: Arc<str>,
    pub params: Vec<Arc<str>>,
    pub body: Arc<Expr>,
    pub env: Arc<RwLock<Env>>,
    pub is_variadic: bool,
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// 実際の等価性比較はValueのPartialEqでArc::ptr_eqを使用
impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        // マクロの等価性は名前、パラメータ、ボディ、可変長フラグで判定
        self.name == other.name
            && self.params == other.params
            && self.body == other.body
            && self.is_variadic == other.is_variadic
    }
}

/// ネイティブ関数
#[derive(Clone)]
pub struct NativeFunc {
    pub name: &'static str,
    pub func: fn(&[Value]) -> Result<Value, String>,
}

impl PartialEq for NativeFunc {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Debug for NativeFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NativeFunc({})", self.name)
    }
}

/// モジュール
#[derive(Debug, Clone)]
pub struct Module {
    pub name: Arc<str>,
    pub file_path: Option<String>,
    pub env: Arc<RwLock<Env>>,
    pub exports: Option<crate::HashSet<Arc<str>>>, // Noneの場合は全公開、Some([])の場合は明示的export（O(1)高速検索）
}

impl Module {
    pub fn new(name: impl Into<Arc<str>>, file_path: Option<String>) -> Self {
        Module {
            name: name.into(),
            file_path,
            env: Arc::new(RwLock::new(Env::new())),
            exports: None, // デフォルトは全公開
        }
    }

    /// シンボルが公開されているかチェック
    pub fn is_exported(&self, name: &str) -> bool {
        match &self.exports {
            None => {
                // exportリストがない = デフォルト全公開
                // ただしprivateフラグがあれば非公開
                self.env
                    .read()
                    .get_binding(name)
                    .map(|b| !b.is_private)
                    .unwrap_or(false)
            }
            Some(set) => {
                // exportリストがある = 明示的export（O(1)検索）
                set.contains(name)
            }
        }
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Arc::ptr_eq(&self.env, &other.env)
    }
}

/// バインディング（値 + メタデータ）
#[derive(Debug, Clone)]
pub struct Binding {
    pub value: Value,
    pub is_private: bool,
}

impl Binding {
    pub fn public(value: Value) -> Self {
        Binding {
            value,
            is_private: false,
        }
    }

    pub fn private(value: Value) -> Self {
        Binding {
            value,
            is_private: true,
        }
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.is_private == other.is_private
    }
}

/// 環境（変数の束縛を保持）
///
/// レキシカルスコープを実装するための環境チェーン。
/// 親環境への参照を持ち、変数探索時に上位スコープを辿ります。
///
/// # 構造
/// - `bindings`: 現在のスコープの変数束縛
/// - `parent`: 親環境への参照（グローバル環境の場合は`None`）
///
/// # スレッドセーフ性
/// - `Arc<RwLock<Env>>`でラップされ、複数スレッドから安全にアクセス可能
/// - 並行処理（goroutine）でのクロージャキャプチャで使用
#[derive(Debug, Clone)]
pub struct Env {
    bindings: crate::HashMap<Arc<str>, Binding>,
    parent: Option<Arc<RwLock<Env>>>,
    /// モジュール名（moduleで設定、未設定ならファイル名のbasename）
    module_name: Option<String>,
    /// 公開シンボルのリスト（Noneなら全公開、Someなら選択公開）
    exports: Option<crate::HashSet<Arc<str>>>,
}

// NOTE: この実装はrust-analyzerの誤検知を防ぐためのもの
// FunctionとMacroがEnvを持つため、それらのPartialEq実装に必要
impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        // 環境はポインタ比較とバインディング比較の組み合わせ
        self.bindings == other.bindings
            && self.module_name == other.module_name
            && self.exports == other.exports
            && match (&self.parent, &other.parent) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Env {
            bindings: crate::new_hashmap(),
            parent: None,
            module_name: None,
            exports: None,
        }
    }

    #[inline]
    pub fn with_parent(parent: Arc<RwLock<Env>>) -> Self {
        Env {
            bindings: crate::new_hashmap(),
            parent: Some(parent),
            module_name: None,
            exports: None,
        }
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings
            .get(name)
            .map(|b| b.value.clone())
            .or_else(|| self.parent.as_ref().and_then(|p| p.read().get(name)))
    }

    pub fn get_binding(&self, name: &str) -> Option<Binding> {
        self.bindings.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.read().get_binding(name))
        })
    }

    pub fn set(&mut self, name: impl Into<Arc<str>>, value: Value) {
        self.bindings.insert(name.into(), Binding::public(value));
    }

    pub fn set_private(&mut self, name: impl Into<Arc<str>>, value: Value) {
        self.bindings.insert(name.into(), Binding::private(value));
    }

    pub fn set_binding(&mut self, name: impl Into<Arc<str>>, binding: Binding) {
        self.bindings.insert(name.into(), binding);
    }

    /// バインディングの反復子を取得（モジュールシステム用）
    pub fn bindings(&self) -> impl Iterator<Item = (&Arc<str>, &Value)> {
        self.bindings.iter().map(|(k, b)| (k, &b.value))
    }

    /// バインディング（メタデータ含む）の反復子を取得
    pub fn all_bindings(&self) -> impl Iterator<Item = (&Arc<str>, &Binding)> {
        self.bindings.iter()
    }

    /// 親環境を取得
    pub fn parent(&self) -> Option<Arc<RwLock<Env>>> {
        self.parent.clone()
    }

    /// ローカルバインディングのみを取得（親環境にあるものを除外）
    ///
    /// SmallVec を使用してスタック上での処理を最適化（通常8個以下）
    pub fn local_bindings(&self) -> smallvec::SmallVec<[(Arc<str>, Binding); 8]> {
        let mut result = smallvec::SmallVec::new();
        for (name, binding) in &self.bindings {
            // 親環境にも存在するキーはスキップ（グローバル変数/関数）
            let is_local = if let Some(ref parent) = self.parent {
                parent.read().get(name).is_none()
            } else {
                true // 親がない場合はすべてローカル
            };

            if is_local {
                result.push((name.clone(), binding.clone()));
            }
        }
        result
    }

    // ========================================
    // モジュールシステム関連
    // ========================================

    /// モジュール名を設定（module宣言で使用）
    pub fn set_module_name(&mut self, name: String) {
        self.module_name = Some(name);
    }

    /// モジュール名を取得
    pub fn module_name(&self) -> Option<&str> {
        self.module_name.as_deref()
    }

    /// 公開シンボルを追加（export宣言で使用）
    pub fn add_exports(&mut self, symbols: Vec<impl Into<Arc<str>>>) {
        if let Some(ref mut exports) = self.exports {
            exports.extend(symbols.into_iter().map(|s| s.into()));
        } else {
            self.exports = Some(symbols.into_iter().map(|s| s.into()).collect());
        }
    }

    /// シンボルが公開されているか確認
    pub fn is_exported(&self, name: &str) -> bool {
        // defn- (is_private=true) は常に非公開
        if let Some(binding) = self.bindings.get(name) {
            if binding.is_private {
                return false;
            }
        }

        // exportsがNoneなら全公開（デフォルト）
        // exportsがSomeなら、リストに含まれるもののみ公開
        match &self.exports {
            None => true,
            Some(exports) => exports.contains(name),
        }
    }

    /// 公開されているバインディングのみを取得
    pub fn exported_bindings(&self) -> Vec<(String, Value)> {
        self.bindings
            .iter()
            .filter(|(name, _)| self.is_exported(name))
            .map(|(name, binding)| (name.to_string(), binding.value.clone()))
            .collect()
    }
}

/// AST（抽象構文木）の式
/// すべてのvariantに位置情報（Span）を持つ
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // リテラル
    Nil {
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    Integer {
        value: i64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    FString {
        parts: Vec<FStringPart>,
        span: Span,
    },
    Symbol {
        name: std::sync::Arc<str>,
        span: Span,
    },
    Keyword {
        name: std::sync::Arc<str>,
        span: Span,
    },

    // コレクション
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Vector {
        items: Vec<Expr>,
        span: Span,
    },
    Map {
        pairs: Vec<(Expr, Expr)>,
        span: Span,
    },

    // 特殊形式
    Def {
        name: std::sync::Arc<str>,
        value: Box<Expr>,
        is_private: bool,
        span: Span,
    },
    Fn {
        params: Vec<Pattern>,
        body: Box<Expr>,
        is_variadic: bool,
        span: Span,
    },
    Let {
        bindings: Vec<(Pattern, Expr)>,
        body: Box<Expr>,
        span: Span,
    },
    If {
        test: Box<Expr>,
        then: Box<Expr>,
        otherwise: Option<Box<Expr>>,
        span: Span,
    },
    Do {
        exprs: Vec<Expr>,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Try {
        expr: Box<Expr>,
        span: Span,
    },
    Defer {
        expr: Box<Expr>,
        span: Span,
    },
    Loop {
        bindings: Vec<(std::sync::Arc<str>, Expr)>,
        body: Box<Expr>,
        span: Span,
    },
    Recur {
        args: Vec<Expr>,
        span: Span,
    },
    When {
        condition: Box<Expr>,
        body: Vec<Expr>,
        span: Span,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Expr>,
        span: Span,
    },
    Until {
        condition: Box<Expr>,
        body: Vec<Expr>,
        span: Span,
    },
    WhileSome {
        binding: std::sync::Arc<str>,
        expr: Box<Expr>,
        body: Vec<Expr>,
        span: Span,
    },
    UntilError {
        binding: std::sync::Arc<str>,
        expr: Box<Expr>,
        body: Vec<Expr>,
        span: Span,
    },

    // マクロ
    Mac {
        name: std::sync::Arc<str>,
        params: Vec<std::sync::Arc<str>>,
        is_variadic: bool,
        body: Box<Expr>,
        span: Span,
    },
    Quasiquote {
        expr: Box<Expr>,
        span: Span,
    },
    Unquote {
        expr: Box<Expr>,
        span: Span,
    },
    UnquoteSplice {
        expr: Box<Expr>,
        span: Span,
    },

    // モジュール
    Module {
        name: Arc<str>,
        span: Span,
    },
    Export {
        symbols: Vec<Arc<str>>,
        span: Span,
    },
    Use {
        module: Arc<str>,
        mode: UseMode,
        span: Span,
    },

    // 関数呼び出し
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    /// 式の位置情報を取得
    pub fn span(&self) -> Span {
        match self {
            Expr::Nil { span } => *span,
            Expr::Bool { span, .. } => *span,
            Expr::Integer { span, .. } => *span,
            Expr::Float { span, .. } => *span,
            Expr::String { span, .. } => *span,
            Expr::FString { span, .. } => *span,
            Expr::Symbol { span, .. } => *span,
            Expr::Keyword { span, .. } => *span,
            Expr::List { span, .. } => *span,
            Expr::Vector { span, .. } => *span,
            Expr::Map { span, .. } => *span,
            Expr::Def { span, .. } => *span,
            Expr::Fn { span, .. } => *span,
            Expr::Let { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Do { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Try { span, .. } => *span,
            Expr::Defer { span, .. } => *span,
            Expr::Loop { span, .. } => *span,
            Expr::Recur { span, .. } => *span,
            Expr::When { span, .. } => *span,
            Expr::While { span, .. } => *span,
            Expr::Until { span, .. } => *span,
            Expr::WhileSome { span, .. } => *span,
            Expr::UntilError { span, .. } => *span,
            Expr::Mac { span, .. } => *span,
            Expr::Quasiquote { span, .. } => *span,
            Expr::Unquote { span, .. } => *span,
            Expr::UnquoteSplice { span, .. } => *span,
            Expr::Module { span, .. } => *span,
            Expr::Export { span, .. } => *span,
            Expr::Use { span, .. } => *span,
            Expr::Call { span, .. } => *span,
        }
    }

    /// ダミーのSpan（位置情報なし）を返す
    /// builtinsなどの動的生成コードで使用
    pub fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    /// シンボルを簡単に作成（ダミーSpan付き）
    pub fn symbol_dummy(name: impl Into<String>) -> Self {
        let name_str = name.into();
        Expr::Symbol {
            name: crate::intern::intern_symbol(&name_str),
            span: Self::dummy_span(),
        }
    }
}

/// useのインポートモード
#[derive(Debug, Clone, PartialEq)]
pub enum UseMode {
    /// :only [sym1 sym2]
    Only(Vec<Arc<str>>),
    /// :as alias
    As(Arc<str>),
    /// :all
    All,
}

/// matchのアーム（パターン -> 結果）
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

/// パターン
#[derive(Debug, Clone, PartialEq)]
/// パターンマッチング（分解束縛）の統一型
///
/// let/fn/matchすべてで共通のパターン構文を使用します。
/// ただし、一部の機能はmatch専用です（実行時チェック）。
pub enum Pattern {
    // ========================================
    // match専用機能（let/fnでは使用不可）
    // ========================================
    /// ワイルドカード（_）- match専用
    Wildcard,

    /// nilリテラル - match専用
    Nil,
    /// bool値リテラル - match専用
    Bool(bool),
    /// 整数リテラル - match専用
    Integer(i64),
    /// 浮動小数点リテラル - match専用
    Float(f64),
    /// 文字列リテラル - match専用
    String(String),
    /// キーワードリテラル - match専用
    Keyword(std::sync::Arc<str>),

    /// 変換 var => expr (束縛後に変換を適用) - match専用
    Transform(std::sync::Arc<str>, Box<Expr>),

    /// Orパターン (p1 | p2 | p3) - match専用
    Or(Vec<Pattern>),

    // ========================================
    // 共通機能（let/fn/matchすべてで使用可能）
    // ========================================
    /// 変数（バインディング）
    Var(std::sync::Arc<str>),

    /// リストパターン (x & rest)
    /// 例: (let [[x & rest] '(1 2 3)] ...) => x=1, rest='(2 3)
    List(Vec<Pattern>, Option<Box<Pattern>>), // (固定部, rest部)

    /// ベクタパターン [x y] or [x & rest]
    /// 例: (let [[x y & rest] [1 2 3 4]] ...) => x=1, y=2, rest='(3 4)
    /// FnParamから統合：restパラメータ追加
    Vector(Vec<Pattern>, Option<Box<Pattern>>), // (固定部, rest部)

    /// マップパターン {:key val} or {:key val :as m}
    /// 例: (let [{:x a :y b :as m} {:x 10 :y 20}] ...) => a=10, b=20, m={:x 10 :y 20}
    /// FnParamから統合：:asパラメータ追加
    Map(
        Vec<(std::sync::Arc<str>, Pattern)>,
        Option<std::sync::Arc<str>>,
    ), // (キー・パターン対, :as変数)

    /// As束縛 pattern :as var
    /// 例: (match [1 2] [x y :as all] -> all) => [1 2]
    As(Box<Pattern>, std::sync::Arc<str>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Keyword(k) => write!(f, ":{}", k),
            Value::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Vector(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Function(_) => write!(f, "#<function>"),
            Value::NativeFunc(nf) => write!(f, "#<native-function:{}>", nf.name),
            Value::Macro(m) => write!(f, "#<macro:{}>", m.name),
            Value::Atom(a) => write!(f, "#<atom:{}>", a.read()),
            Value::Channel(_) => write!(f, "#<channel>"),
            Value::Scope(_) => write!(f, "#<scope>"),
            Value::Stream(_) => write!(f, "#<stream>"),
            Value::Uvar(id) => write!(f, "#<uvar:{}>", id),
        }
    }
}
