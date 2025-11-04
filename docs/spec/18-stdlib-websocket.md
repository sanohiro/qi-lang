# 18. WebSocket通信

QiはWebSocketクライアント機能を標準ライブラリとして提供します。リアルタイム通信が必要なアプリケーション（チャット、通知、ライブデータ更新など）で活用できます。

## 18.1 基本的な使い方

### WebSocket接続の確立

```qi
;; WebSocketサーバーに接続
(def conn (ws/connect "ws://localhost:8080/ws"))

;; TLS対応（wss://）
(def conn (ws/connect "wss://echo.websocket.org"))
```

`ws/connect`は接続IDを返します。この接続IDを使って、メッセージの送受信や接続のクローズを行います。

### メッセージの送信

```qi
;; テキストメッセージを送信
(ws/send conn "Hello, WebSocket!")

;; JSONメッセージを送信
(ws/send conn (json/stringify {:type "chat" :msg "Hello"}))
```

### メッセージの受信

```qi
;; メッセージを受信（ブロッキング）
(def msg (ws/receive conn))

;; メッセージタイプに応じて処理
(match (get msg :type)
  "message" -> (println (get msg :data))
  "close" -> (println "接続が閉じられました")
  "error" -> (println (get msg :error)))
```

### 接続のクローズ

```qi
;; 接続をクローズ
(ws/close conn)
```

## 18.2 メッセージタイプ

`ws/receive`は以下のいずれかのタイプのメッセージマップを返します：

### テキストメッセージ

```qi
{:type "message" :data "受信したテキスト"}
```

### バイナリメッセージ

```qi
{:type "binary" :data "base64エンコードされたデータ"}
```

バイナリデータはbase64エンコードされた文字列として返されます。

### クローズメッセージ

```qi
{:type "close" :code 1000 :reason "Normal closure"}
```

サーバーが接続をクローズした場合に受信します。

### エラー

```qi
{:type "error" :error "エラーメッセージ"}
```

通信エラーが発生した場合に受信します。

## 18.3 実用的なパターン

### チャットクライアント

```qi
(defn chat-client [url username]
  (let [conn (ws/connect url)]
    (if (error? conn)
      (println f"接続エラー: {conn}")
      (do
        ;; 参加メッセージを送信
        (ws/send conn (json/stringify {:type "join" :user username}))

        ;; メッセージ受信ループ
        (loop []
          (let [msg (ws/receive conn)]
            (match (get msg :type)
              "message" ->
                (do
                  (let [data (json/parse (get msg :data))]
                    (println f"{(get data \"user\")}: {(get data \"msg\")}"))
                  (recur))
              "close" -> (println "接続が切断されました")
              "error" -> (println f"エラー: {(get msg :error)}")
              _ -> (recur))))

        (ws/close conn)))))

;; 使用例
(chat-client "ws://localhost:8080/chat" "Alice")
```

### JSONメッセージの送受信

```qi
;; JSON送信ヘルパー
(defn send-json [conn data]
  (ws/send conn (json/stringify data)))

;; JSON受信ヘルパー
(defn receive-json [conn]
  (let [response (ws/receive conn)]
    (match (get response :type)
      "message" -> {:ok (json/parse (get response :data))}
      _ -> {:error response})))

;; 使用例
(def conn (ws/connect "ws://localhost:8080/api"))
(send-json conn {:action "subscribe" :channel "news"})
(def msg (receive-json conn))
```

### エラーハンドリング

```qi
(defn safe-ws-connect [url max-retries]
  (loop [retries 0]
    (let [conn (try (ws/connect url))]
      (if (error? conn)
        (if (< retries max-retries)
          (do
            (println f"接続失敗。リトライ {(+ retries 1)}/{max-retries}")
            (recur (+ retries 1)))
          (do
            (println "最大リトライ回数に達しました")
            nil))
        conn))))

;; 使用例
(def conn (safe-ws-connect "ws://localhost:8080" 3))
```

### タイムアウト付き受信

WebSocketは標準ではブロッキングですが、goroutineとchannelを使ってタイムアウトを実装できます：

```qi
(defn ws-receive-timeout [conn timeout-ms]
  (let [ch (go/chan)
        timeout-ch (go/timeout timeout-ms)]
    ;; 受信をgoroutineで実行
    (go
      (let [msg (ws/receive conn)]
        (go/send ch msg)))
    ;; タイムアウトまたは受信を待つ
    (go/select
      ch -> (go/recv ch)
      timeout-ch -> {:error "timeout"})))
```

## 18.4 接続管理

### 複数接続の管理

接続IDは整数なので、マップで複数の接続を管理できます：

```qi
(def connections (atom {}))

(defn add-connection [name url]
  (let [conn (ws/connect url)]
    (if (not (error? conn))
      (do
        (swap! connections assoc name conn)
        {:ok conn})
      {:error conn})))

(defn send-to [name message]
  (if-let [conn (get @connections name)]
    (ws/send conn message)
    (println f"接続 {name} が見つかりません")))

;; 使用例
(add-connection :server1 "ws://server1.com/ws")
(add-connection :server2 "ws://server2.com/ws")
(send-to :server1 "Hello from server1")
```

## 18.5 サーバーサイド実装

現在、QiはWebSocketクライアント機能のみを提供しています。サーバーサイドのWebSocket機能は将来のバージョンで追加予定です。

## 18.6 関数リファレンス

### ws/connect

```qi
(ws/connect url) ;=> connection-id
```

WebSocketサーバーに接続します。

- **引数**: `url` - WebSocket URL（ws://またはwss://）
- **戻り値**: 接続ID（Integer）
- **エラー**: 接続失敗時はエラー文字列

**例**:
```qi
(def conn (ws/connect "wss://echo.websocket.org"))
```

### ws/send

```qi
(ws/send conn-id message) ;=> nil
```

WebSocketでメッセージを送信します。

- **引数**:
  - `conn-id` - 接続ID
  - `message` - 送信するメッセージ（文字列）
- **戻り値**: nil
- **エラー**: 送信失敗時はエラーを投げる

**例**:
```qi
(ws/send conn "Hello, WebSocket!")
```

### ws/receive

```qi
(ws/receive conn-id) ;=> message-map
```

WebSocketからメッセージを受信します。メッセージが届くまでブロックします。

- **引数**: `conn-id` - 接続ID
- **戻り値**: メッセージマップ（`:type`, `:data`, `:code`, `:reason`, `:error`）
- **エラー**: 受信失敗時はエラーを投げる

**例**:
```qi
(def msg (ws/receive conn))
(println (get msg :type)) ;=> "message"
```

### ws/close

```qi
(ws/close conn-id) ;=> nil
```

WebSocket接続をクローズします。

- **引数**: `conn-id` - 接続ID
- **戻り値**: nil
- **エラー**: クローズ失敗時はエラーを投げる

**例**:
```qi
(ws/close conn)
```

## 18.7 ベストプラクティス

### 1. エラーハンドリング

WebSocket接続は不安定になる可能性があるため、適切なエラーハンドリングを実装してください：

```qi
(defn handle-ws-message [conn]
  (try
    (let [msg (ws/receive conn)]
      (match (get msg :type)
        "message" -> (process-message msg)
        "error" -> (log-error msg)
        "close" -> (reconnect)))
    |>? (fn [result]
          (if (error? result)
            (handle-error result)
            result))))
```

### 2. 再接続ロジック

ネットワークの問題で接続が切れた場合、自動的に再接続する仕組みを実装してください：

```qi
(defn auto-reconnect [url]
  (loop [delay 1000]
    (let [conn (try (ws/connect url))]
      (if (error? conn)
        (do
          (println f"再接続を {delay}ms 後に試行...")
          (go/sleep delay)
          (recur (min (* delay 2) 30000))) ;; exponential backoff
        conn))))
```

### 3. リソースのクリーンアップ

使用後は必ず接続をクローズしてください：

```qi
(defn with-websocket [url f]
  (let [conn (ws/connect url)]
    (if (error? conn)
      {:error conn}
      (try
        (f conn)
        |> (fn [result]
             (ws/close conn)
             result)))))
```

## 18.8 セキュリティ

### TLS/SSL

機密情報を扱う場合は、必ず`wss://`（WebSocket Secure）を使用してください：

```qi
;; ✅ 良い例
(ws/connect "wss://api.example.com/secure")

;; ❌ 悪い例（機密情報を扱う場合）
(ws/connect "ws://api.example.com/secure")
```

### 認証

WebSocket接続時に認証トークンを含める場合は、URLクエリパラメータまたは最初のメッセージで送信します：

```qi
;; URLクエリパラメータで認証
(def conn (ws/connect f"wss://api.example.com/ws?token={token}"))

;; 最初のメッセージで認証
(def conn (ws/connect "wss://api.example.com/ws"))
(ws/send conn (json/stringify {:type "auth" :token token}))
```

## 18.9 パフォーマンス

### Ping/Pongの自動処理

QiのWebSocket実装は、Ping/Pongフレームを自動的に処理します。ユーザーは特別な対応を行う必要はありません。

### バイナリデータ

バイナリメッセージはbase64エンコードされて返されます。大量のバイナリデータを扱う場合は、エンコード/デコードのオーバーヘッドに注意してください。

## まとめ

QiのWebSocket機能により、リアルタイム通信が簡単に実装できます：

- `ws/connect` - 接続の確立
- `ws/send` - メッセージ送信
- `ws/receive` - メッセージ受信（ブロッキング）
- `ws/close` - 接続のクローズ

エラーハンドリングと再接続ロジックを適切に実装することで、安定したリアルタイムアプリケーションを構築できます。
