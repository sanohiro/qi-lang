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
pub mod value;

// ========================================
// システム共通型定義
// ========================================

use rustc_hash::FxBuildHasher;

/// Qi専用のHashMap型（im::HashMapにFxHasherを適用）
///
/// FxHasherは非暗号学的ハッシュで、デフォルトのSipHashより高速。
/// 将来的にハッシュアルゴリズムを変更する場合もここだけ修正すればOK。
pub type HashMap<K, V> = im::HashMap<K, V, FxBuildHasher>;

/// Qi専用HashMapを作成するヘルパー関数
#[inline]
pub fn new_hashmap<K, V>() -> HashMap<K, V> {
    im::HashMap::with_hasher(FxBuildHasher)
}

/// Qi専用のHashSet型（im::HashSetにFxHasherを適用）
pub type HashSet<T> = im::HashSet<T, FxBuildHasher>;

/// Qi専用HashSetを作成するヘルパー関数
#[inline]
pub fn new_hashset<T>() -> HashSet<T> {
    im::HashSet::with_hasher(FxBuildHasher)
}
