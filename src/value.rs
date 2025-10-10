use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// f-stringの部品（文字列またはコード）
#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String),   // 通常の文字列部分
    Code(String),   // {expr} 内のコード
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
    /// シンボル
    Symbol(String),
    /// キーワード（:keyのような形式）
    Keyword(String),
    /// リスト（cons cell）
    List(Vec<Value>),
    /// ベクタ
    Vector(Vec<Value>),
    /// マップ
    Map(HashMap<String, Value>),
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
    /// ユニーク変数（マクロの衛生性）
    Uvar(u64),
}

/// チャネル（送信・受信両方可能）
#[derive(Debug, Clone)]
pub struct Channel {
    pub sender: Sender<Value>,
    pub receiver: Receiver<Value>,
}

/// スコープ（Structured Concurrency用）
#[derive(Debug, Clone)]
pub struct Scope {
    pub cancelled: Arc<RwLock<bool>>,
}

impl Value {
    /// 真偽値判定（nilとfalse以外はすべてtruthy）
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

/// 関数の定義
#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Expr,
    pub env: Env,
    pub is_variadic: bool, // &argsに対応
}

/// マクロの定義
#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expr,
    pub env: Env,
    pub is_variadic: bool,
}

/// ネイティブ関数
#[derive(Clone)]
pub struct NativeFunc {
    pub name: String,
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

/// 環境（変数の束縛を保持）
#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Arc<RwLock<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<RwLock<Env>>) -> Self {
        Env {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned().or_else(|| {
            self.parent.as_ref().and_then(|p| p.read().get(name))
        })
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// バインディングの反復子を取得（モジュールシステム用）
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.bindings.iter()
    }
}

/// AST（抽象構文木）の式
#[derive(Debug, Clone)]
pub enum Expr {
    // リテラル
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    FString(Vec<FStringPart>),
    Symbol(String),
    Keyword(String),

    // コレクション
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),

    // 特殊形式
    Def(String, Box<Expr>),
    Fn {
        params: Vec<String>,
        body: Box<Expr>,
        is_variadic: bool,
    },
    Let {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },
    If {
        test: Box<Expr>,
        then: Box<Expr>,
        otherwise: Option<Box<Expr>>,
    },
    Do(Vec<Expr>),
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Try(Box<Expr>),
    Defer(Box<Expr>),
    Loop {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },
    Recur(Vec<Expr>),

    // マクロ
    Mac {
        name: String,
        params: Vec<String>,
        is_variadic: bool,
        body: Box<Expr>,
    },
    Quasiquote(Box<Expr>),
    Unquote(Box<Expr>),
    UnquoteSplice(Box<Expr>),

    // モジュール
    Module(String),
    Export(Vec<String>),
    Use {
        module: String,
        mode: UseMode,
    },

    // 関数呼び出し
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
}

/// useのインポートモード
#[derive(Debug, Clone)]
pub enum UseMode {
    /// :only [sym1 sym2]
    Only(Vec<String>),
    /// :as alias
    As(String),
    /// :all
    All,
}

/// matchのアーム（パターン -> 結果）
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

/// パターン
#[derive(Debug, Clone)]
pub enum Pattern {
    /// ワイルドカード（_）
    Wildcard,
    /// nil
    Nil,
    /// bool値
    Bool(bool),
    /// 整数リテラル
    Integer(i64),
    /// 浮動小数点リテラル
    Float(f64),
    /// 文字列リテラル
    String(String),
    /// キーワードリテラル
    Keyword(String),
    /// 変数（バインディング）
    Var(String),
    /// リストパターン [x, ...rest]
    List(Vec<Pattern>, Option<Box<Pattern>>), // (固定部, 可変部)
    /// ベクタパターン [x, y]
    Vector(Vec<Pattern>),
    /// マップパターン {:key val}
    Map(Vec<(String, Pattern)>),
    /// As束縛 pattern :as var
    As(Box<Pattern>, String),
    /// 変換 var => expr (束縛後に変換を適用)
    Transform(String, Box<Expr>),
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
            Value::Uvar(id) => write!(f, "#<uvar:{}>", id),
        }
    }
}
