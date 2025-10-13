//! プロファイラー - 実行時間測定とパフォーマンス分析

use crate::i18n::{fmt_ui_msg, ui_msg, UiMsg};
use crate::value::Value;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

/// プロファイルデータ
#[derive(Debug, Clone)]
pub struct ProfileData {
    pub call_count: usize,
    pub total_time: Duration,
}

/// グローバルプロファイラー
pub struct Profiler {
    enabled: bool,
    data: HashMap<String, ProfileData>,
}

impl Profiler {
    fn new() -> Self {
        Profiler {
            enabled: false,
            data: HashMap::new(),
        }
    }
}

/// グローバルプロファイラーインスタンス
fn profiler() -> &'static Mutex<Profiler> {
    static PROFILER: OnceLock<Mutex<Profiler>> = OnceLock::new();
    PROFILER.get_or_init(|| Mutex::new(Profiler::new()))
}

/// プロファイリングを開始
pub fn native_profile_start(_args: &[Value]) -> Result<Value, String> {
    profiler().lock().enabled = true;
    Ok(Value::Nil)
}

/// プロファイリングを停止
pub fn native_profile_stop(_args: &[Value]) -> Result<Value, String> {
    profiler().lock().enabled = false;
    Ok(Value::Nil)
}

/// プロファイルデータをクリア
pub fn native_profile_clear(_args: &[Value]) -> Result<Value, String> {
    let mut p = profiler().lock();
    p.data.clear();
    Ok(Value::Nil)
}

/// プロファイル結果をレポート
pub fn native_profile_report(_args: &[Value]) -> Result<Value, String> {
    let p = profiler().lock();

    if p.data.is_empty() {
        println!("{}", ui_msg(UiMsg::ProfileNoData));
        println!("{}", ui_msg(UiMsg::ProfileUseStart));
        return Ok(Value::Nil);
    }

    // データを収集してソート（実行時間の降順）
    let mut entries: Vec<_> = p.data.iter().collect();
    entries.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));

    println!("\n{}", ui_msg(UiMsg::ProfileReport));
    println!("{}", ui_msg(UiMsg::ProfileTableHeader));
    println!("{}", "=".repeat(82));

    let mut total_time = Duration::ZERO;
    for (name, data) in &entries {
        let avg_micros = if data.call_count > 0 {
            data.total_time.as_micros() / data.call_count as u128
        } else {
            0
        };

        println!(
            "{:<40} {:>10} {:>15.3} {:>15}",
            name,
            data.call_count,
            data.total_time.as_secs_f64() * 1000.0,
            avg_micros
        );

        total_time += data.total_time;
    }

    println!("{}", "=".repeat(82));
    println!(
        "{}",
        fmt_ui_msg(
            UiMsg::ProfileTotalTime,
            &[&format!("{:.3}", total_time.as_secs_f64() * 1000.0)],
        )
    );

    Ok(Value::Nil)
}

/// 関数実行を記録（eval.rsから呼び出される）
pub fn record_call(name: &str, duration: Duration) {
    let mut p = profiler().lock();
    if !p.enabled {
        return;
    }

    let entry = p.data.entry(name.to_string()).or_insert(ProfileData {
        call_count: 0,
        total_time: Duration::ZERO,
    });

    entry.call_count += 1;
    entry.total_time += duration;
}

/// プロファイリングが有効かチェック
pub fn is_enabled() -> bool {
    profiler().lock().enabled
}
