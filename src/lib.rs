pub mod builtins;
pub mod constants;
#[cfg(feature = "dap-server")]
pub mod dap;
pub mod debugger;
pub mod eval;
pub mod i18n;
pub mod intern;
pub mod lexer;
pub mod parser;
pub mod project;
pub mod upgrade;
pub mod value;

// ========================================
// システム共通型定義
// ========================================

use ahash::RandomState;

/// Qi専用のHashMap型（im::HashMapにahashを適用）
///
/// ahashは高速な非暗号学的ハッシュで、FxHasherよりさらに高速（SIMD最適化）。
/// 将来的にハッシュアルゴリズムを変更する場合もここだけ修正すればOK。
pub type HashMap<K, V> = im::HashMap<K, V, RandomState>;

/// Qi専用HashMapを作成するヘルパー関数
#[inline]
pub fn new_hashmap<K, V>() -> HashMap<K, V> {
    im::HashMap::with_hasher(RandomState::new())
}

/// Qi専用のHashSet型（im::HashSetにahashを適用）
pub type HashSet<T> = im::HashSet<T, RandomState>;

/// Qi専用HashSetを作成するヘルパー関数
#[inline]
pub fn new_hashset<T>() -> HashSet<T> {
    im::HashSet::with_hasher(RandomState::new())
}
