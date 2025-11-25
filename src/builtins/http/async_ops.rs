use super::*;
use crate::builtins::util::kw;

/// HTTP GETリクエスト (非同期)
pub fn native_get_async(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["http/get-async"]));
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/get-async", "URL"])),
    };

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: Arc::new(parking_lot::Mutex::new(Some(sender.clone()))),
        receiver,
    });

    rayon::spawn(move || {
        let result = simple::native_get(&[Value::String(url)]);
        let response = match result {
            Ok(value) => {
                // 成功: {:ok value}
                let mut map = crate::new_hashmap();
                map.insert(kw("ok"), value);
                Value::Map(map)
            }
            Err(err_msg) => {
                // エラー: {:error message}
                let mut map = crate::new_hashmap();
                map.insert(kw("error"), Value::String(err_msg));
                Value::Map(map)
            }
        };
        let _ = sender.send(response);
    });

    Ok(Value::Channel(result_channel))
}

/// HTTP POSTリクエスト (非同期)
pub fn native_post_async(args: &[Value]) -> Result<Value, String> {
    crate::check_args!(args, 2, "http/post-async");

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeString, &["http/post-async", "URL"])),
    };

    let body = args[1].clone();

    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(crate::value::Channel {
        sender: Arc::new(parking_lot::Mutex::new(Some(sender.clone()))),
        receiver,
    });

    rayon::spawn(move || {
        let result = simple::native_post(&[Value::String(url), body]);
        let response = match result {
            Ok(value) => {
                // 成功: {:ok value}
                let mut map = crate::new_hashmap();
                map.insert(kw("ok"), value);
                Value::Map(map)
            }
            Err(err_msg) => {
                // エラー: {:error message}
                let mut map = crate::new_hashmap();
                map.insert(kw("error"), Value::String(err_msg));
                Value::Map(map)
            }
        };
        let _ = sender.send(response);
    });

    Ok(Value::Channel(result_channel))
}
