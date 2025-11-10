# 標準ライブラリ - コマンド実行（cmd/）

**外部コマンドとシェルスクリプトの実行**

すべての関数は `cmd/` モジュールに属します。

---

## 概要

cmdモジュールは、Qi言語から外部コマンドやシェルスクリプトを実行するための関数群を提供します。パイプライン演算子（`|>`）と統合されており、データの流れとしてコマンド実行を扱えます。

**主な機能**:
- コマンド実行と終了コード取得
- シェルスクリプト実行（sh/cmd.exe経由）
- 標準入出力制御
- パイプライン統合（Qi → コマンド → Qi）
- ストリーム処理（行単位・バイト単位）
- 双方向インタラクティブプロセス

**クロスプラットフォーム対応**:
- **Unix/Linux/macOS**: `/bin/sh` 経由でシェルコマンド実行
- **Windows**: `cmd.exe` 経由でコマンド実行

**注意**: このモジュールは `cmd-exec` feature でコンパイルされます。

---

## 基本実行

### cmd/exec - コマンド実行（終了コードを返す）

**引数**: コマンド（文字列 or ベクタ）
**戻り値**: 終了コード（整数）
**エラー**: コマンドが見つからない場合

```qi
;; シェル経由（文字列）
(cmd/exec "ls -la")                    ;; => 0
(cmd/exec "false")                     ;; => 1

;; 直接実行（ベクタ）
(cmd/exec ["ls" "-la"])                ;; => 0
(cmd/exec ["git" "status"])            ;; => 0

;; 終了コードでのエラー判定
(let [code (cmd/exec "test -f data.txt")]
  (if (= code 0)
    (println "ファイルが存在します")
    (println "ファイルが存在しません")))
```

**シェル vs 直接実行の違い**:
- **文字列**: シェル経由（パイプ、リダイレクト、環境変数展開が可能）
- **ベクタ**: 直接実行（シェルインジェクション対策、高速）

---

## シェル実行

### cmd/sh - シェルコマンド実行（簡易版）

**引数**: コマンド文字列
**戻り値**: 終了コード（整数）

```qi
;; Unix/Linux/macOS: /bin/sh 経由
(cmd/sh "ls -la | grep .qi")           ;; => 0

;; Windows: cmd.exe /C 経由
(cmd/sh "dir *.qi")                    ;; => 0

;; パイプライン、リダイレクトが使える
(cmd/sh "cat *.txt | sort | uniq > result.txt")  ;; => 0
(cmd/sh "curl -s https://example.com | grep title")  ;; => 0

;; 複数コマンド実行
(cmd/sh "cd build && make clean && make")  ;; => 0
```

### cmd/sh! - シェルコマンド実行（詳細版）

**引数**: コマンド文字列
**戻り値**: `{:stdout "..." :stderr "..." :exit 0}` マップ

```qi
;; 標準出力・標準エラー・終了コードを全て取得
(cmd/sh! "cat *.txt | grep pattern | wc -l")
;; => {:stdout "      42\n" :stderr "" :exit 0}

;; エラー時の詳細情報
(cmd/sh! "ls non-existent-file")
;; => {:stdout ""
;;     :stderr "ls: non-existent-file: No such file or directory\n"
;;     :exit 1}

;; 結果を分解して使用
(let [result (cmd/sh! "git status --porcelain")
      stdout (get result "stdout")
      exit (get result "exit")]
  (if (= exit 0)
    (if (str/blank? stdout)
      (println "作業ディレクトリはクリーンです")
      (println f"変更あり:\n{stdout}"))
    (println "gitリポジトリではありません")))
```

---

## パイプライン統合

### cmd/pipe - コマンドに標準入力を渡す

**引数**: コマンド、[入力データ（文字列 or リスト）]
**戻り値**: 標準出力（文字列）
**エラー**: 終了コードが0以外の場合

```qi
;; コマンド単独実行
(cmd/pipe "ls -la")
;; => "total 48\ndrwxr-xr-x  5 user  staff  160 Jan  1 12:00 .\n..."

;; パイプラインで入力を渡す
("hello\nworld\n" |> (cmd/pipe "sort"))
;; => "hello\nworld\n"

;; リストを渡す（各要素が行になる）
(["line1" "line2" "line3"] |> (cmd/pipe "wc -l"))
;; => "       3\n"

;; 実用例: JSONをjqで処理
(http/get "https://api.example.com/data")
  |> (get _ "body")
  |> (cmd/pipe "jq '.users[] | .name'")
  |> str/lines
  |> (map str/trim)
;; => ["Alice" "Bob" "Charlie"]

;; 実用例: データ整形パイプライン
(io/read-lines "data.csv")
  |> (map (fn [line] (str/split line ",")))
  |> (filter (fn [row] (> (len row) 2)))
  |> (map (fn [row] (join row "\t")))
  |> (fn [lines] (join lines "\n"))
  |> (cmd/pipe "sort -t $'\t' -k2,2")
  |> println
```

**失敗時の動作**:
```qi
;; コマンドが失敗するとエラーを投げる
(try
  (cmd/pipe "grep pattern" "no match here")
  (catch e
    (println f"コマンド失敗: {e}")))
;; => "コマンド失敗: Command failed with exit code 1: ..."
```

### cmd/pipe! - コマンド実行（詳細版）

**引数**: コマンド、[入力データ]
**戻り値**: `[stdout stderr exitcode]` ベクタ

```qi
;; 標準出力・標準エラー・終了コードを全て取得
(cmd/pipe! "cat test.txt")
;; => ["file content\n" "" 0]

;; 分割代入で使用
(let [[out err code] (cmd/pipe! "ls -la")]
  (if (= code 0)
    (println out)
    (println f"エラー: {err}")))

;; パイプラインで入力を渡す
(["test"] |> (cmd/pipe! ["wc" "-l"]))
;; => ["       1\n" "" 0]

;; 実用例: ビルドツール実行
(defn build [target]
  (let [[out err code] (cmd/pipe! f"cargo build --release --bin {target}")]
    (if (= code 0)
      {:status :ok :output out}
      {:status :error :message err})))

(build "qi-lang")
;; => {:status :ok :output "   Compiling qi-lang v0.1.0\n..."}
```

### cmd/lines - テキストを行のリストに分割（ヘルパー）

**引数**: テキスト
**戻り値**: 行のリスト

```qi
;; コマンド出力を行に分割
("a\nb\nc" |> cmd/lines)
;; => ["a" "b" "c"]

;; パイプラインでの実用例
(cmd/pipe "ls -1")
  |> cmd/lines
  |> (filter (fn [f] (str/ends-with? f ".qi")))
  |> (map (fn [f] (str/replace f ".qi" "")))
;; => ["main" "lib" "test"]
```

---

## ストリーム処理

### cmd/stream-lines - 行単位ストリーム

**引数**: コマンド（文字列 or ベクタ）
**戻り値**: Stream（各要素は行文字列）

```qi
;; ログファイルをストリームとして処理
(def log-stream (cmd/stream-lines "tail -f /var/log/app.log"))

;; 先頭10行を取得
(log-stream |> (stream/take 10) |> realize)

;; フィルタリングしながら処理
(cmd/stream-lines "cat large.log")
  |> (stream/filter (fn [line] (str/contains? line "ERROR")))
  |> (stream/take 100)
  |> realize
  |> (map println)

;; 実用例: リアルタイムログ監視
(defn watch-errors [logfile]
  (cmd/stream-lines f"tail -f {logfile}")
    |> (stream/filter (fn [line] (str/contains? line "ERROR")))
    |> (stream/map (fn [line]
         (let [timestamp (time/now)]
           {:time timestamp :message line})))
    |> (stream/each send-alert))

(watch-errors "/var/log/app.log")
```

### cmd/stream-bytes - バイト単位ストリーム

**引数**: コマンド、[チャンクサイズ（デフォルト4096）]
**戻り値**: Stream（各要素はBase64エンコードされたバイト列）

```qi
;; 大きなファイルをチャンク単位で処理
(cmd/stream-bytes "cat large-file.bin")
  |> (stream/take 10)  ;; 最初の10チャンク（40KB）
  |> realize

;; カスタムチャンクサイズ（8KB）
(cmd/stream-bytes "curl -L https://example.com/video.mp4" 8192)
  |> (stream/each process-chunk)

;; 実用例: ダウンロード進捗表示
(defn download-with-progress [url output]
  (let [total (atom 0)]
    (cmd/stream-bytes f"curl -L {url}" 4096)
      |> (stream/each (fn [chunk]
           (let [size (len chunk)]
             (swap! total (fn [t] (+ t size)))
             (println f"Downloaded: {@total} bytes"))))
      |> (stream/reduce str/concat "")
      |> (fn [data] (io/write-file data output))))

(download-with-progress "https://example.com/file.zip" "/tmp/download.zip")
```

---

## インタラクティブプロセス

### cmd/interactive - 双方向プロセス起動

**引数**: コマンド（文字列 or ベクタ）
**戻り値**: プロセスハンドル（Map形式）

```qi
;; Pythonインタプリタを起動
(def py (cmd/interactive "python3 -i"))

;; コマンドを送信
(cmd/write py "print(1+1)\n")

;; 結果を読み取り
(cmd/read-line py)  ;; => "2"

;; プロセスを終了
(cmd/write py "exit()\n")
(cmd/wait py)       ;; => {:exit 0 :stderr ""}
```

### cmd/write - プロセスに書き込み

**引数**: プロセスハンドル、データ（文字列）
**戻り値**: nil

```qi
;; REPLに複数行送信
(cmd/write py "def greet(name):\n")
(cmd/write py "    return f'Hello, {name}!'\n")
(cmd/write py "\n")
(cmd/write py "print(greet('Qi'))\n")
```

### cmd/read-line - プロセスから1行読み取り

**引数**: プロセスハンドル
**戻り値**: 読み取った行（文字列）、EOFなら`nil`

```qi
;; 結果を読み取り
(cmd/read-line py)  ;; => "Hello, Qi!"

;; 全ての出力を読み取り
(defn read-all [proc]
  (loop [lines []]
    (let [line (cmd/read-line proc)]
      (if (some? line)
        (recur (conj lines line))
        lines))))

(read-all py)  ;; => ["line1" "line2" "line3"]
```

### cmd/wait - プロセス終了を待つ

**引数**: プロセスハンドル
**戻り値**: `{:exit exit_code :stderr "..."}`

```qi
;; プロセスを終了して結果を取得
(cmd/write py "exit()\n")
(cmd/wait py)
;; => {:exit 0 :stderr ""}
```

### 実用例: インタラクティブシェル

```qi
(defn run-script [script-lines]
  (let [proc (cmd/interactive "python3 -i")]
    ;; スクリプトを実行
    (script-lines
     |> (map (fn [line] (cmd/write proc (str line "\n"))))
     |> realize)

    ;; 結果を収集
    (let [results (loop [acc []]
                    (let [line (cmd/read-line proc)]
                      (if (some? line)
                        (recur (conj acc line))
                        acc)))]
      ;; プロセス終了
      (cmd/write proc "exit()\n")
      (cmd/wait proc)
      results)))

(run-script ["print(1+1)" "print('hello')" "print(2*3)"])
;; => ["2" "hello" "6"]
```

---

## 実用例

### ビルドツール統合

```qi
;; Cargoビルド実行
(defn cargo-build [target]
  (let [[out err code] (cmd/pipe! f"cargo build --release --bin {target}")]
    (if (= code 0)
      (do
        (println "ビルド成功!")
        (println out)
        :ok)
      (do
        (println "ビルド失敗:")
        (println err)
        :error))))

(cargo-build "qi-lang")
```

### Git操作

```qi
;; Gitステータス確認
(defn git-status []
  (let [result (cmd/sh! "git status --porcelain")]
    (if (= (get result "exit") 0)
      (let [stdout (get result "stdout")]
        (if (str/blank? stdout)
          {:clean true :files []}
          {:clean false :files (str/lines stdout)}))
      {:error (get result "stderr")})))

(git-status)
;; => {:clean false :files ["M src/main.rs" "?? new-file.txt"]}

;; Git変更をコミット
(defn git-commit [message]
  (do
    (cmd/sh "git add .")
    (let [code (cmd/exec f"git commit -m '{message}'")]
      (if (= code 0)
        (println "コミット成功!")
        (println "コミット失敗（変更なし？）")))))

(git-commit "feat: Add new feature")
```

### データ処理パイプライン

```qi
;; CSVをSQLiteで処理
(defn process-csv-with-sqlite [csv-file query]
  (let [db "/tmp/temp.db"]
    ;; CSVをインポート
    (cmd/sh f"sqlite3 {db} '.mode csv' '.import {csv-file} data'")

    ;; SQLクエリ実行
    (cmd/pipe f"sqlite3 -csv {db} \"{query}\"")
      |> str/lines
      |> (map (fn [line] (str/split line ",")))
      |> (map (fn [row] (zipmap ["id" "name" "value"] row)))))

(process-csv-with-sqlite "data.csv" "SELECT * FROM data WHERE value > 100")
;; => [{:id "1" :name "Alice" :value "120"} ...]
```

### システム監視

```qi
;; ディスク使用量監視
(defn check-disk-usage []
  (cmd/pipe "df -h /")
    |> str/lines
    |> (drop 1)  ;; ヘッダーをスキップ
    |> first
    |> (str/split _ " ")
    |> (filter (fn [s] (not (str/blank? s))))
    |> (fn [cols] {:filesystem (nth cols 0)
                   :size (nth cols 1)
                   :used (nth cols 2)
                   :avail (nth cols 3)
                   :percent (nth cols 4)}))

(check-disk-usage)
;; => {:filesystem "/dev/disk1s1" :size "931Gi" :used "450Gi"
;;     :avail "481Gi" :percent "49%"}

;; CPU使用率監視
(defn watch-cpu []
  (cmd/stream-lines "top -l 0 -s 1")
    |> (stream/filter (fn [line] (str/contains? line "CPU usage")))
    |> (stream/map (fn [line]
         (let [parts (str/split line " ")]
           {:user (nth parts 2) :sys (nth parts 4)})))
    |> (stream/take 10)
    |> realize)

(watch-cpu)
;; => [{:user "15.5%" :sys "8.2%"} {:user "12.3%" :sys "6.1%"} ...]
```

### テストスクリプト実行

```qi
;; テストを実行して結果を集計
(defn run-tests []
  (let [[out err code] (cmd/pipe! "cargo test -- --nocapture")]
    {:passed (str/contains? out "test result: ok")
     :output out
     :errors err
     :exit-code code}))

(run-tests)
;; => {:passed true :output "running 15 tests\n..." :errors "" :exit-code 0}
```

### マルチプラットフォームコマンド

```qi
;; OSに応じたコマンド実行
(defn list-processes []
  (if (= (os/platform) "windows")
    (cmd/pipe "tasklist")
    (cmd/pipe "ps aux"))
  |> str/lines)

(list-processes)
;; Unix: => ["USER       PID %CPU %MEM ..." "root         1  0.0  0.1 ..." ...]
;; Windows: => ["Image Name           PID Session Name ..." "System Idle Process   0 ..." ...]
```

---

## エラーハンドリング

### コマンド失敗のハンドリング

```qi
;; try-catchでエラーを捕捉
(try
  (cmd/pipe "grep pattern file.txt")
  (catch e
    (println f"検索失敗: {e}")
    nil))

;; pipe!で終了コードを確認
(let [[out err code] (cmd/pipe! "test -f data.txt")]
  (if (= code 0)
    (println "ファイルが存在します")
    (println "ファイルが存在しません")))

;; sh!でエラーメッセージを取得
(let [result (cmd/sh! "ls non-existent")]
  (if (= (get result "exit") 0)
    (get result "stdout")
    (do
      (println f"エラー: {(get result 'stderr')}")
      nil)))
```

### タイムアウト処理

```qi
;; タイムアウト付き実行（外部コマンド利用）
(defn exec-with-timeout [cmd timeout-sec]
  (let [timeout-cmd (if (= (os/platform) "windows")
                       f"timeout /t {timeout-sec} && {cmd}"
                       f"timeout {timeout-sec} {cmd}")]
    (cmd/sh! timeout-cmd)))

(exec-with-timeout "sleep 10" 5)
;; => {:stdout "" :stderr "timeout: killed" :exit 124}
```

---

## セキュリティ考慮事項

### コマンドインジェクション対策

```qi
;; ❌ 危険: ユーザー入力を直接シェルに渡す
(defn bad-search [pattern file]
  (cmd/pipe f"grep {pattern} {file}"))  ;; インジェクション可能!

(bad-search "test; rm -rf /" "data.txt")  ;; 危険!

;; ✅ 安全: ベクタで渡す（シェル経由しない）
(defn safe-search [pattern file]
  (cmd/pipe ["grep" pattern file]))

(safe-search "test; rm -rf /" "data.txt")  ;; 安全（リテラル文字列として扱われる）

;; ✅ 安全: エスケープ処理
(defn escape-shell-arg [s]
  (str "'" (str/replace s "'" "'\\''") "'"))

(defn safe-search-shell [pattern file]
  (cmd/pipe f"grep {(escape-shell-arg pattern)} {(escape-shell-arg file)}"))

(safe-search-shell "test; rm -rf /" "data.txt")  ;; 安全
```

### パス検証

```qi
;; ❌ 危険: パストラバーサル攻撃
(defn bad-read-file [filename]
  (cmd/pipe f"cat {filename}"))

(bad-read-file "../../../etc/passwd")  ;; 危険!

;; ✅ 安全: パス検証
(defn safe-read-file [filename base-dir]
  (let [fullpath (path/join base-dir filename)
        canonical (path/canonicalize fullpath)]
    (if (str/starts-with? canonical base-dir)
      (cmd/pipe ["cat" canonical])
      (throw "Invalid path"))))

(safe-read-file "../../../etc/passwd" "/var/data")  ;; エラー
(safe-read-file "file.txt" "/var/data")  ;; OK
```

### 権限管理

```qi
;; ✅ 最小権限で実行
(defn run-sandboxed [cmd]
  ;; ユーザー権限で実行（sudoを使わない）
  (cmd/sh cmd))

;; ❌ 危険: sudo/管理者権限での実行は避ける
;; (cmd/sh "sudo rm -rf /")  ;; 絶対にやらない!
```

---

## 関数一覧

### 基本実行
- `cmd/exec` - コマンド実行（終了コードを返す）
- `cmd/sh` - シェルコマンド実行（簡易版）
- `cmd/sh!` - シェルコマンド実行（詳細版、stdout/stderr/exit）

### パイプライン統合
- `cmd/pipe` - コマンドに標準入力を渡す（stdoutを返す）
- `cmd/pipe!` - コマンド実行（[stdout stderr exit]を返す）
- `cmd/lines` - テキストを行のリストに分割

### ストリーム処理
- `cmd/stream-lines` - コマンドのstdoutを行単位でストリーム化
- `cmd/stream-bytes` - コマンドのstdoutをバイト単位でストリーム化

### インタラクティブプロセス
- `cmd/interactive` - 双方向プロセスを起動
- `cmd/write` - プロセスのstdinに書き込み
- `cmd/read-line` - プロセスのstdoutから1行読み取り
- `cmd/wait` - プロセス終了を待つ

---

## パフォーマンス最適化

### 大量のコマンド実行

```qi
;; ❌ 遅い: シェル経由で毎回実行
(files |> (map (fn [f] (cmd/sh f"wc -l {f}"))))

;; ✅ 速い: 1つのコマンドにまとめる
(files |> (fn [fs] (join fs " ")) |> (cmd/pipe "wc -l"))

;; ✅ 速い: 並列実行
(files |> (pmap (fn [f] (cmd/exec ["wc" "-l" f]))))
```

### ストリームでのメモリ効率化

```qi
;; ❌ メモリ非効率: 全データを一度に読み込み
(cmd/pipe "cat large-file.txt")
  |> str/lines
  |> (map process-line)

;; ✅ メモリ効率的: ストリームで処理
(cmd/stream-lines "cat large-file.txt")
  |> (stream/map process-line)
  |> (stream/take 1000)
  |> realize
```

---

## デバッグ

```qi
;; コマンド実行のトレース
(defn trace-cmd [cmd]
  (do
    (println f"実行: {cmd}")
    (let [[out err code] (cmd/pipe! cmd)]
      (println f"終了コード: {code}")
      (println f"stdout: {out}")
      (println f"stderr: {err}")
      [out err code])))

(trace-cmd "ls -la")

;; 環境変数の確認
(cmd/pipe "env")
  |> str/lines
  |> (filter (fn [line] (str/starts-with? line "PATH=")))
  |> (map println)
```
