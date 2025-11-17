use super::msg_key::MsgKey;
use super::ui_msg::UiMsg;
use super::{EN_MSGS, EN_UI_MSGS, JA_MSGS, JA_UI_MSGS};
use std::sync::OnceLock;

/// 言語設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ja,
}

impl Lang {
    /// 環境変数から言語を取得
    /// 優先順位: QI_LANG > LANG > デフォルト(en)
    pub fn from_env() -> Self {
        // QI_LANGが設定されていればそれを優先
        if let Ok(lang) = std::env::var("QI_LANG") {
            return Self::parse(&lang);
        }

        // LANGから言語コードを取得（ja_JP.UTF-8 -> ja）
        if let Ok(lang) = std::env::var("LANG") {
            let lang_code = lang.split('_').next().unwrap_or("");
            return Self::parse(lang_code);
        }

        // デフォルトは英語
        Lang::En
    }

    /// 言語コードをパース
    fn parse(code: &str) -> Self {
        match code {
            "ja" | "ja_JP" => Lang::Ja,
            "en" | "en_US" | "en_GB" => Lang::En,
            _ => Lang::En, // 未対応言語は英語にフォールバック
        }
    }

    /// 言語コードを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
        }
    }
}

/// メッセージマネージャー（HashMap検索、enフォールバック）
pub struct Messages {
    lang: Lang,
}

impl Messages {
    /// 言語設定でMessagesインスタンスを作成
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// メッセージを取得（jaになければenにフォールバック）
    pub fn get(&self, key: MsgKey) -> &'static str {
        match self.lang {
            Lang::En => EN_MSGS.get(&key).unwrap_or(&"[missing message]"),
            Lang::Ja => JA_MSGS
                .get(&key)
                .or_else(|| EN_MSGS.get(&key))
                .unwrap_or(&"[missing message]"),
        }
    }

    /// UIメッセージを取得（en以外で対象LANGになければenにフォールバック）
    pub fn ui(&self, key: UiMsg) -> &'static str {
        match self.lang {
            Lang::En => EN_UI_MSGS.get(&key).unwrap_or(&"[missing message]"),
            Lang::Ja => JA_UI_MSGS
                .get(&key)
                .or_else(|| EN_UI_MSGS.get(&key))
                .unwrap_or(&"[missing message]"),
        }
    }

    /// メッセージをフォーマット（プレースホルダー {0}, {1}, ... を置換）
    /// 一度の走査でO(n)で処理
    pub fn fmt(&self, key: MsgKey, args: &[&str]) -> String {
        let template = self.get(key);

        // 予想サイズを確保（テンプレート + 引数の合計長）
        let estimated_size = template.len() + args.iter().map(|s| s.len()).sum::<usize>();
        let mut result = String::with_capacity(estimated_size);

        let mut chars = template.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // プレースホルダーの可能性
                let mut digits = String::new();
                let mut temp_chars = chars.clone();

                // 数字を収集
                while let Some(d) = temp_chars.next() {
                    if d.is_ascii_digit() {
                        digits.push(d);
                    } else if d == '}' && !digits.is_empty() {
                        // 正常なプレースホルダー
                        if let Ok(index) = digits.parse::<usize>() {
                            if let Some(arg) = args.get(index) {
                                result.push_str(arg);
                                chars = temp_chars;
                                break;
                            }
                        }
                        // パース失敗時はそのまま出力
                        result.push(ch);
                        break;
                    } else {
                        // プレースホルダーではない
                        result.push(ch);
                        break;
                    }
                }

                if digits.is_empty() {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// UIメッセージをフォーマット
    /// 一度の走査でO(n)で処理
    pub fn fmt_ui(&self, key: UiMsg, args: &[&str]) -> String {
        let template = self.ui(key);

        // 予想サイズを確保（テンプレート + 引数の合計長）
        let estimated_size = template.len() + args.iter().map(|s| s.len()).sum::<usize>();
        let mut result = String::with_capacity(estimated_size);

        let mut chars = template.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // プレースホルダーの可能性
                let mut digits = String::new();
                let mut temp_chars = chars.clone();

                // 数字を収集
                while let Some(d) = temp_chars.next() {
                    if d.is_ascii_digit() {
                        digits.push(d);
                    } else if d == '}' && !digits.is_empty() {
                        // 正常なプレースホルダー
                        if let Ok(index) = digits.parse::<usize>() {
                            if let Some(arg) = args.get(index) {
                                result.push_str(arg);
                                chars = temp_chars;
                                break;
                            }
                        }
                        // パース失敗時はそのまま出力
                        result.push(ch);
                        break;
                    } else {
                        // プレースホルダーではない
                        result.push(ch);
                        break;
                    }
                }

                if digits.is_empty() {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

// グローバルインスタンス
static MESSAGES: OnceLock<Messages> = OnceLock::new();

/// i18nシステムを初期化
pub fn init() {
    // OnceLockで自動初期化されるため、ここでは何もしない
    // ただし、初期化を強制したい場合は messages() を呼ぶ
    let _ = messages();
}

/// グローバルなメッセージインスタンスを取得
pub fn messages() -> &'static Messages {
    MESSAGES.get_or_init(|| Messages::new(Lang::from_env()))
}

/// メッセージを取得してフォーマット
pub fn fmt_msg(key: MsgKey, args: &[&str]) -> String {
    messages().fmt(key, args)
}

/// UIメッセージを取得してフォーマット
pub fn fmt_ui_msg(key: UiMsg, args: &[&str]) -> String {
    messages().fmt_ui(key, args)
}

/// メッセージを取得
pub fn msg(key: MsgKey) -> &'static str {
    messages().get(key)
}

/// UIメッセージを取得
pub fn ui_msg(key: UiMsg) -> &'static str {
    messages().ui(key)
}
