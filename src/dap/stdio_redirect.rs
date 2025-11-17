//! 標準入出力のリダイレクト機能

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;

#[cfg(windows)]
use super::{Event, OutputEventBody, EVENT_OUTPUT, MSG_TYPE_EVENT};

    #[cfg(unix)]
    type NativeHandle = i32;

    // 統一された構造体定義
    #[cfg(unix)]
    pub struct StdioRedirect {
        original_stdout: NativeHandle,
        original_stderr: NativeHandle,
        stdout_read: NativeHandle,
        stderr_read: NativeHandle,
    }

    // Windows版はSendHandle使用
    #[cfg(windows)]
    pub struct StdioRedirect {
        original_stdout: platform::SendHandle,
        original_stderr: platform::SendHandle,
        stdout_read: platform::SendHandle,
        stderr_read: platform::SendHandle,
    }

    // プラットフォーム固有のヘルパー関数
    #[cfg(unix)]
    mod platform {
        use super::NativeHandle;
        use std::io;

        pub const STDOUT_NO: NativeHandle = libc::STDOUT_FILENO;
        pub const STDERR_NO: NativeHandle = libc::STDERR_FILENO;

        /// ファイルディスクリプタを複製
        ///
        /// SAFETY: libc::dup()はPOSIXシステムコールで、有効なファイルディスクリプタに
        /// 対して安全に実行できます。エラーは負の値で返されチェックします。
        pub unsafe fn dup(handle: NativeHandle) -> io::Result<NativeHandle> {
            let new_handle = libc::dup(handle);
            if new_handle < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(new_handle)
            }
        }

        /// ファイルディスクリプタを閉じる
        ///
        /// SAFETY: libc::close()はPOSIXシステムコールで、有効なファイルディスクリプタを
        /// 閉じます。既に閉じられたfdに対して複数回呼び出すと未定義動作になりますが、
        /// このモジュールではfdの所有権を適切に管理しているため、二重closeは発生しません。
        pub unsafe fn close(handle: NativeHandle) {
            libc::close(handle);
        }

        /// パイプを作成
        ///
        /// SAFETY: libc::pipe()はPOSIXシステムコールで、新しいパイプを作成します。
        /// pipe配列は有効なメモリ領域で、サイズも正しいです。エラーは負の値で返されチェックします。
        pub unsafe fn create_pipe() -> io::Result<(NativeHandle, NativeHandle)> {
            let mut pipe: [i32; 2] = [0, 0];
            if libc::pipe(pipe.as_mut_ptr()) < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok((pipe[0], pipe[1]))
            }
        }

        /// ファイルディスクリプタをリダイレクト
        ///
        /// SAFETY: libc::dup2()はPOSIXシステムコールで、new_handleをstd_noに複製します。
        /// 両方のfdが有効であることが前提です。エラーは負の値で返されチェックします。
        pub unsafe fn redirect(new_handle: NativeHandle, std_no: NativeHandle) -> io::Result<()> {
            if libc::dup2(new_handle, std_no) < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        /// ハンドルを複製（リーダー用）
        ///
        /// SAFETY: libc::dup()はPOSIXシステムコールで、有効なファイルディスクリプタに
        /// 対して安全に実行できます。エラーは負の値で返され、Noneとして扱います。
        pub unsafe fn dup_for_reader(handle: NativeHandle) -> Option<NativeHandle> {
            let new_handle = libc::dup(handle);
            if new_handle < 0 {
                None
            } else {
                Some(new_handle)
            }
        }

        /// ハンドルからFileリーダーを作成
        ///
        /// SAFETY: from_raw_fd()は生のfdからFileを作成します。
        /// handleは有効なfdである必要があり、所有権はFileに移動します。
        /// Fileがdropされるときに自動的にcloseされるため、二重closeは発生しません。
        pub unsafe fn create_reader(handle: NativeHandle) -> std::fs::File {
            use std::os::unix::io::FromRawFd;
            std::fs::File::from_raw_fd(handle)
        }
    }

    #[cfg(windows)]
    pub(super) mod platform {
        use std::io;
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Console::*;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        /// Windows HANDLEをSend-safeにするラッパー
        #[derive(Clone, Copy)]
        pub struct SendHandle(pub HANDLE);

        // SAFETY: SendHandleはWindows HANDLEのラッパーで、スレッド間で安全に送信できます。
        // Windows HANDLEはプロセス全体で有効な識別子であり、複数スレッドから同時にアクセスしても
        // カーネルレベルで同期が保証されています。
        unsafe impl Send for SendHandle {}
        // SAFETY: SendHandleは複数スレッドから同時に参照しても安全です。
        // Windows APIのハンドル操作はスレッドセーフです。
        unsafe impl Sync for SendHandle {}

        pub const STDOUT_NO: u32 = STD_OUTPUT_HANDLE;
        pub const STDERR_NO: u32 = STD_ERROR_HANDLE;

        /// 標準ハンドルを取得
        ///
        /// SAFETY: GetStdHandle()はWindows APIで、標準入出力ハンドルを取得します。
        /// INVALID_HANDLE_VALUEのチェックを行い、エラーは適切に処理されます。
        pub unsafe fn get_std_handle(handle_id: u32) -> io::Result<SendHandle> {
            let handle = GetStdHandle(handle_id);
            if handle == INVALID_HANDLE_VALUE {
                Err(io::Error::last_os_error())
            } else {
                Ok(SendHandle(handle))
            }
        }

        /// ハンドルを閉じる
        ///
        /// SAFETY: CloseHandle()はWindows APIで、有効なハンドルを閉じます。
        /// 既に閉じられたハンドルに対して複数回呼び出すと未定義動作になりますが、
        /// このモジュールではハンドルの所有権を適切に管理しているため、二重closeは発生しません。
        pub unsafe fn close(handle: SendHandle) {
            CloseHandle(handle.0);
        }

        /// パイプを作成
        ///
        /// SAFETY: CreatePipe()はWindows APIで、新しいパイプを作成します。
        /// ハンドルポインタは有効なメモリ領域です。エラーは0で返されチェックします。
        pub unsafe fn create_pipe() -> io::Result<(SendHandle, SendHandle)> {
            use windows_sys::Win32::System::Pipes::CreatePipe;
            let mut read_handle: HANDLE = std::ptr::null_mut();
            let mut write_handle: HANDLE = std::ptr::null_mut();
            if CreatePipe(&mut read_handle, &mut write_handle, std::ptr::null(), 0) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok((SendHandle(read_handle), SendHandle(write_handle)))
            }
        }

        /// 標準ハンドルをリダイレクト
        ///
        /// SAFETY: SetStdHandle()はWindows APIで、標準ハンドルを設定します。
        /// new_handleは有効なハンドルである必要があります。エラーは0で返されチェックします。
        pub unsafe fn redirect(new_handle: SendHandle, std_handle_id: u32) -> io::Result<()> {
            if SetStdHandle(std_handle_id, new_handle.0) == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        /// ハンドルを複製（リーダー用）
        ///
        /// SAFETY: DuplicateHandle()はWindows APIで、有効なハンドルを複製します。
        /// エラーは0で返され、Noneとして扱います。
        pub unsafe fn dup_for_reader(handle: SendHandle) -> Option<SendHandle> {
            let mut dup_handle: HANDLE = std::ptr::null_mut();
            if DuplicateHandle(
                GetCurrentProcess(),
                handle.0,
                GetCurrentProcess(),
                &mut dup_handle,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ) == 0
            {
                None
            } else {
                Some(SendHandle(dup_handle))
            }
        }

        /// ハンドルからFileリーダーを作成
        ///
        /// SAFETY: from_raw_handle()は生のハンドルからFileを作成します。
        /// handleは有効なハンドルである必要があり、所有権はFileに移動します。
        /// Fileがdropされるときに自動的にcloseされるため、二重closeは発生しません。
        pub unsafe fn create_reader(handle: SendHandle) -> std::fs::File {
            use std::os::windows::io::FromRawHandle;
            std::fs::File::from_raw_handle(handle.0 as _)
        }
    }

    impl StdioRedirect {
        /// stdout/stderrをパイプにリダイレクト（Unix版）
        #[cfg(unix)]
        pub fn new() -> io::Result<Self> {
            use platform::*;

            // SAFETY: この関数全体がunsafeです。以下のplatform関数を使用しています:
            // - dup(): stdoutとstderrのfdを複製して保存します。
            // - create_pipe(): 新しいパイプを作成します。
            // - redirect(): stdoutとstderrを新しいパイプにリダイレクトします。
            // - close(): 不要になったfdを閉じます。
            // すべてのfdは有効性がチェックされており、エラー時には適切にクリーンアップされます。
            // fdの所有権は構造体で管理され、dropで自動的に元に戻されます。
            unsafe {
                // 元のstdout/stderrを保存
                let original_stdout = dup(STDOUT_NO)?;
                let original_stderr = dup(STDERR_NO).inspect_err(|_e| {
                    close(original_stdout);
                })?;

                // パイプ作成（stdout、stderr）
                let (stdout_read, stdout_write) = create_pipe().inspect_err(|_e| {
                    close(original_stdout);
                    close(original_stderr);
                })?;

                let (stderr_read, stderr_write) = create_pipe().inspect_err(|_e| {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stdout_write);
                })?;

                // リダイレクト
                if let Err(e) = redirect(stdout_write, STDOUT_NO) {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stdout_write);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                if let Err(e) = redirect(stderr_write, STDERR_NO) {
                    close(original_stdout);
                    close(original_stderr);
                    close(stdout_read);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                // 書き込み側を閉じる
                close(stdout_write);
                close(stderr_write);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read,
                    stderr_read,
                })
            }
        }

        /// stdout/stderrをパイプにリダイレクト（Windows版）
        #[cfg(windows)]
        pub fn new() -> io::Result<Self> {
            use platform::*;

            // SAFETY: この関数全体がunsafeです。以下のplatform関数を使用しています:
            // - get_std_handle(): stdoutとstderrのハンドルを取得して保存します。
            // - create_pipe(): 新しいパイプを作成します。
            // - redirect(): stdoutとstderrを新しいパイプにリダイレクトします。
            // - close(): 不要になったハンドルを閉じます。
            // すべてのハンドルは有効性がチェックされており、エラー時には適切にクリーンアップされます。
            // ハンドルの所有権は構造体で管理され、dropで自動的に元に戻されます。
            unsafe {
                // 元のstdout/stderrを保存
                let original_stdout = get_std_handle(STDOUT_NO)?;
                let original_stderr = get_std_handle(STDERR_NO)?;

                // パイプ作成（stdout、stderr）
                let (stdout_read, stdout_write) = create_pipe()?;

                let (stderr_read, stderr_write) = create_pipe().map_err(|e| {
                    close(stdout_read);
                    close(stdout_write);
                    e
                })?;

                // リダイレクト
                if let Err(e) = redirect(stdout_write, STDOUT_NO) {
                    close(stdout_read);
                    close(stdout_write);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                if let Err(e) = redirect(stderr_write, STDERR_NO) {
                    close(stdout_read);
                    close(stderr_read);
                    close(stderr_write);
                    return Err(e);
                }

                // 書き込み側を閉じる
                close(stdout_write);
                close(stderr_write);

                Ok(Self {
                    original_stdout,
                    original_stderr,
                    stdout_read,
                    stderr_read,
                })
            }
        }

        /// stdoutパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stdout_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(event_tx, seq_base, "stdout", self.stdout_read)
        }

        /// stderrパイプから読み取ってDAPイベントを送信するタスクを起動
        pub fn spawn_stderr_reader(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
        ) -> tokio::task::JoinHandle<()> {
            self.spawn_reader_impl(event_tx, seq_base, "stderr", self.stderr_read)
        }

        /// パイプから読み取ってDAPイベントを送信する共通実装
        #[cfg(unix)]
        fn spawn_reader_impl(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
            category: &'static str,
            read_handle: NativeHandle,
        ) -> tokio::task::JoinHandle<()> {
            // ハンドルを複製
            // SAFETY: dup_for_reader()は有効なfdを複製します。
            // read_handleはnew()で作成された有効なパイプのfdです。
            let read_dup = unsafe { platform::dup_for_reader(read_handle) };

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;

                // 複製に失敗した場合は早期リターン
                let Some(handle) = read_dup else {
                    return;
                };

                // リーダーを作成
                // SAFETY: create_reader()は複製されたfdからFileを作成します。
                // handleは有効なfdで、所有権はFileに移動します。
                let file = unsafe { platform::create_reader(handle) };
                let async_file = tokio::fs::File::from_std(file);
                let reader = tokio::io::BufReader::new(async_file);
                let mut lines = reader.lines();

                // 行ごとに読み取ってDAPイベントを送信
                let mut seq = seq_base;
                while let Ok(Some(line)) = lines.next_line().await {
                    let event = format!(
                        r#"{{"seq":{},"type":"event","event":"output","body":{{"category":"{}","output":"{}"}}}}"#,
                        seq,
                        category,
                        line.replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                    );
                    if event_tx.send(event).await.is_err() {
                        break;
                    }
                    seq += 1;
                }

                // パイプを閉じる
                // SAFETY: close()は有効なfdを閉じます。
                // handleはこのスコープで所有されており、二重closeは発生しません。
                unsafe { platform::close(handle) };
            })
        }

        /// パイプから読み取ってDAPイベントを送信する共通実装（Windows版）
        #[cfg(windows)]
        fn spawn_reader_impl(
            &self,
            event_tx: tokio::sync::mpsc::Sender<String>,
            seq_base: i64,
            category: &'static str,
            read_handle: platform::SendHandle,
        ) -> tokio::task::JoinHandle<()> {
            // ハンドルを複製
            // SAFETY: dup_for_reader()は有効なハンドルを複製します。
            // read_handleはnew()で作成された有効なパイプのハンドルです。
            let read_dup = unsafe { platform::dup_for_reader(read_handle) };

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;

                // 複製に失敗した場合は早期リターン
                let Some(handle) = read_dup else {
                    return;
                };

                // リーダーを作成
                // SAFETY: create_reader()は複製されたハンドルからFileを作成します。
                // handleは有効なハンドルで、所有権はFileに移動します。
                let file = unsafe { platform::create_reader(handle) };
                let async_file = tokio::fs::File::from_std(file);
                let mut reader = tokio::io::BufReader::new(async_file);

                let mut line = String::new();
                let mut seq = seq_base;
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let output_msg = OutputEventBody {
                                category: category.to_string(),
                                output: line.clone(),
                            };
                            let output_event = Event {
                                seq,
                                msg_type: MSG_TYPE_EVENT.to_string(),
                                event: EVENT_OUTPUT.to_string(),
                                body: serde_json::to_value(&output_msg).ok(),
                            };
                            if let Ok(event_json) = serde_json::to_string(&output_event) {
                                let _ = event_tx.send(event_json).await;
                            }
                            seq += 1;
                        }
                        Err(_) => break,
                    }
                }
            })
        }
    }

    impl Drop for StdioRedirect {
        /// stdout/stderrを元に戻す
        fn drop(&mut self) {
            use platform::*;

            // SAFETY: drop時にstdout/stderrを元の状態に復元します。
            // - redirect(): 元のfdまたはハンドルを標準出力/エラー出力に復元します。
            // - close(): 保存していたfd/ハンドルを閉じます。
            // すべてのfd/ハンドルはnew()で作成された有効なものです。
            // この時点でそれぞれのfd/ハンドルは一度だけcloseされるため、二重closeは発生しません。
            unsafe {
                // 元のstdout/stderrを復元
                let _ = redirect(self.original_stdout, STDOUT_NO);
                let _ = redirect(self.original_stderr, STDERR_NO);

                // ハンドルをクローズ
                close(self.original_stdout);
                close(self.original_stderr);
                close(self.stdout_read);
                close(self.stderr_read);
            }
        }
    }
