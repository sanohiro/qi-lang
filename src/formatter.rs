//! QiコードフォーマッターのパブリックAPI。
//!
//! 1. `tokenizer` でトリビア付きトークン列を生成
//! 2. `doc` でコメントや空白を保持した Doc ツリーへ変換
//! 3. `layout` で Doc ツリーから最終的なソースコード文字列を生成
//!
//! 現段階ではレイアウトエンジンは元のレイアウトを保つことを重視しており、
//! コメントや文字列を破壊しないための基盤として利用できる。
//! スタイルガイドに沿った整形は今後 `layout` 層で順次実装する。

pub mod doc;
pub mod layout;
pub mod tokenizer;

pub use layout::{format_source, load_config, LayoutConfig};

/// 入力ソースをデフォルト設定でフォーマットする。
pub fn format_with_defaults(source: &str) -> Result<String, String> {
    format_source(source, &LayoutConfig::default())
}
