#!/bin/bash
# Qi標準ライブラリドキュメント生成スクリプト
# すべての.qiドキュメントファイルを生成します

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCS_JA="$PROJECT_ROOT/std/docs/ja"
DOCS_EN="$PROJECT_ROOT/std/docs/en"
BUILTINS_SRC="$PROJECT_ROOT/src/builtins"

# ディレクトリを作成
mkdir -p "$DOCS_JA" "$DOCS_EN"

echo "=== Qi Standard Library Documentation Generator ==="
echo "Project root: $PROJECT_ROOT"
echo "Generating documentation files..."

# 既に作成されたファイル
echo "✓ core.qi (ja/en) - 87 functions"
echo "✓ string.qi (ja) - 72 functions"

# 残りのファイルのスケルトンを生成
cat > "$DOCS_JA/async.qi" << 'EOF'
;; 標準ライブラリドキュメント - 並行処理
;; Async/Concurrency Functions (13 functions - go/*)

(def __doc__go/chan
  {:desc "チャネルを作成します。引数なしで無制限バッファ、引数ありでバッファサイズ指定。"
   :params [{:name "capacity" :type "integer" :desc "バッファサイズ（省略可）"}]
   :returns {:type "channel" :desc "チャネル"}
   :examples ["(def ch (go/chan))      ;; 無制限バッファ"
              "(def ch (go/chan 10))   ;; バッファサイズ10"]})

(def __doc__go/send!
  {:desc "チャネルに値を送信します。"
   :params [{:name "ch" :type "channel" :desc "チャネル"}
            {:name "value" :type "any" :desc "送信する値"}]
   :returns {:type "any" :desc "送信した値"}
   :examples ["(go/send! ch 42)"
              "(go/send! ch {:status :ok})"]})

(def __doc__go/recv!
  {:desc "チャネルから値を受信します（ブロッキング）。"
   :params [{:name "ch" :type "channel" :desc "チャネル"}
            {:name ":timeout" :type "keyword" :desc "タイムアウトキーワード（省略可）"}
            {:name "ms" :type "integer" :desc "タイムアウト時間（ミリ秒、省略可）"}]
   :returns {:type "any" :desc "受信した値（タイムアウト時はnil）"}
   :examples ["(go/recv! ch)"
              "(go/recv! ch :timeout 1000)  ;; 1秒でタイムアウト"]})

(def __doc__go/try-recv!
  {:desc "チャネルから値を非ブロッキング受信します。"
   :params [{:name "ch" :type "channel" :desc "チャネル"}]
   :returns {:type "any|nil" :desc "受信した値（値がない場合nil）"}
   :examples ["(go/try-recv! ch)"]})

(def __doc__go/close!
  {:desc "チャネルをクローズします。"
   :params [{:name "ch" :type "channel" :desc "チャネル"}]
   :returns {:type "nil" :desc "常にnil"}
   :examples ["(go/close! ch)"]})

(def __doc__go/await
  {:desc "Promise（チャネル）の値を待機します。"
   :params [{:name "promise" :type "channel" :desc "Promise（チャネル）"}]
   :returns {:type "any" :desc "Promise の値"}
   :examples ["(def p (go (+ 1 2)))"
              "(go/await p) ;=> 3"]})

(def __doc__go/all
  {:desc "すべてのPromiseが完了するまで待機します。"
   :params [{:name "promises" :type "vector" :desc "Promiseのベクター"}]
   :returns {:type "vector" :desc "すべての結果のベクター"}
   :examples ["(go/all [(go (+ 1 2)) (go (* 3 4))]) ;=> [3 12]"]})

(def __doc__go/race
  {:desc "最初に完了したPromiseの値を返します。"
   :params [{:name "promises" :type "vector" :desc "Promiseのベクター"}]
   :returns {:type "any" :desc "最初に完了したPromiseの値"}
   :examples ["(go/race [(go (sleep 100) :slow) (go :fast)]) ;=> :fast"]})

(def __doc__go/fan-out
  {:desc "値をn個のチャネルに送信します。"
   :params [{:name "value" :type "any" :desc "送信する値"}
            {:name "n" :type "integer" :desc "チャネル数"}]
   :returns {:type "vector" :desc "チャネルのベクター"}
   :examples ["(def chs (go/fan-out 42 3))"
              "(map go/recv! chs) ;=> [42 42 42]"]})

(def __doc__go/fan-in
  {:desc "複数のチャネルから値を集約します。"
   :params [{:name "channels" :type "vector" :desc "チャネルのベクター"}]
   :returns {:type "channel" :desc "集約されたチャネル"}
   :examples ["(def combined (go/fan-in [ch1 ch2 ch3]))"
              "(go/recv! combined)"]})

(def __doc__go/make-scope
  {:desc "キャンセル可能なスコープを作成します。"
   :params []
   :returns {:type "scope" :desc "スコープオブジェクト"}
   :examples ["(def scope (go/make-scope))"]})

(def __doc__go/cancel!
  {:desc "スコープをキャンセルします。"
   :params [{:name "scope" :type "scope" :desc "スコープ"}]
   :returns {:type "nil" :desc "常にnil"}
   :examples ["(go/cancel! scope)"]})

(def __doc__go/cancelled?
  {:desc "スコープがキャンセルされているかを判定します。"
   :params [{:name "scope" :type "scope" :desc "スコープ"}]
   :returns {:type "bool" :desc "キャンセルされている場合true"}
   :examples ["(go/cancelled? scope) ;=> false"
              "(go/cancel! scope)"
              "(go/cancelled? scope) ;=> true"]})
EOF

cat > "$DOCS_JA/http.qi" << 'EOF'
;; 標準ライブラリドキュメント - HTTPクライアント
;; HTTP Client Functions (11 functions)

(def __doc__http/get
  {:desc "HTTP GETリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "map" :desc "レスポンスマップ（:status, :headers, :body）"}
   :examples ["(http/get \"https://api.example.com/data\")"
              "(http/get \"http://localhost:3000/api/users\")"]})

(def __doc__http/post
  {:desc "HTTP POSTリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}
            {:name "body" :type "any" :desc "リクエストボディ（文字列またはマップ）"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/post \"https://api.example.com/users\" {:name \"Alice\"})"
              "(http/post url (json/stringify data))"]})

(def __doc__http/put
  {:desc "HTTP PUTリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}
            {:name "body" :type "any" :desc "リクエストボディ"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/put \"https://api.example.com/users/1\" {:name \"Bob\"})"]})

(def __doc__http/delete
  {:desc "HTTP DELETEリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/delete \"https://api.example.com/users/1\")"]})

(def __doc__http/patch
  {:desc "HTTP PATCHリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}
            {:name "body" :type "any" :desc "リクエストボディ"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/patch \"https://api.example.com/users/1\" {:email \"new@example.com\"})"]})

(def __doc__http/head
  {:desc "HTTP HEADリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "map" :desc "レスポンスマップ（ボディなし）"}
   :examples ["(http/head \"https://example.com\")"]})

(def __doc__http/options
  {:desc "HTTP OPTIONSリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/options \"https://api.example.com\")"]})

(def __doc__http/request
  {:desc "詳細なHTTPリクエストを送信します。"
   :params [{:name "options" :type "map" :desc "リクエストオプション（:method, :url, :headers, :body, :timeout等）"}]
   :returns {:type "map" :desc "レスポンスマップ"}
   :examples ["(http/request {:method :get :url \"https://example.com\" :headers {:Authorization \"Bearer token\"}})"
              "(http/request {:method :post :url url :body data :timeout 5000})"]})

(def __doc__http/get-async
  {:desc "非同期HTTP GETリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "channel" :desc "レスポンスのPromise"}
   :examples ["(def p (http/get-async \"https://api.example.com/data\"))"
              "(go/await p)"]})

(def __doc__http/post-async
  {:desc "非同期HTTP POSTリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}
            {:name "body" :type "any" :desc "リクエストボディ"}]
   :returns {:type "channel" :desc "レスポンスのPromise"}
   :examples ["(def p (http/post-async url data))"
              "(go/await p)"]})

(def __doc__http/get-stream
  {:desc "ストリーミングHTTP GETリクエストを送信します。"
   :params [{:name "url" :type "string" :desc "URL"}]
   :returns {:type "stream" :desc "レスポンスストリーム"}
   :examples ["(def stream (http/get-stream \"https://example.com/large-file\"))"
              "(stream/realize stream 10)"]})
EOF

echo "✓ async.qi (ja) - 13 functions"
echo "✓ http.qi (ja) - 11 functions"

echo ""
echo "=== Summary ==="
echo "Created:"
echo "  - std/docs/ja/core.qi (87 functions)"
echo "  - std/docs/en/core.qi (87 functions)"
echo "  - std/docs/ja/string.qi (72 functions)"
echo "  - std/docs/ja/async.qi (13 functions)"
echo "  - std/docs/ja/http.qi (11 functions)"
echo ""
echo "Remaining files should be created manually or with AI assistance:"
echo "  - server.qi, io.qi, path.qi, env.qi, args.qi, data.qi, cmd.qi, test.qi"
echo "  - time.qi, math.qi, stats.qi, log.qi, profile.qi, stream.qi, db.qi"
echo "  - ds.qi, zip.qi, markdown.qi, util.qi, fn.qi, list.qi, map.qi, set.qi"
echo ""
echo "Total: 27 modules × 2 languages = 54 files"
echo "Status: 5 files created, 49 remaining"
EOF

chmod +x "$PROJECT_ROOT/scripts/generate_std_docs.sh"
