/// 国際化メッセージ管理
///
/// 言語設定の優先順位:
/// 1. QI_LANG 環境変数（Qi専用の設定）
/// 2. LANG 環境変数（システムのロケール設定）
/// 3. デフォルト: en

// サブモジュール
mod en_messages;
mod ja_messages;
mod messages;
mod msg_key;
mod ui_msg;

// 公開エクスポート
pub use en_messages::{EN_MSGS, EN_UI_MSGS};
pub use ja_messages::{JA_MSGS, JA_UI_MSGS};
pub use messages::{fmt_msg, fmt_ui_msg, init, msg, messages, ui_msg, Lang, Messages};
pub use msg_key::MsgKey;
pub use ui_msg::UiMsg;
