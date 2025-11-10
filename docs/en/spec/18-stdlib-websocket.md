# 18. WebSocket Communication

Qi provides WebSocket client functionality as part of its standard library. It can be used for applications requiring real-time communication (chat, notifications, live data updates, etc.).

## 18.1 Basic Usage

### Establishing WebSocket Connection

```qi
;; Connect to WebSocket server
(def conn (ws/connect "ws://localhost:8080/ws"))

;; TLS support (wss://)
(def conn (ws/connect "wss://echo.websocket.org"))
```

`ws/connect` returns a connection ID. This connection ID is used to send/receive messages and close the connection.

### Sending Messages

```qi
;; Send text message
(ws/send conn "Hello, WebSocket!")

;; Send JSON message
(ws/send conn (json/stringify {:type "chat" :msg "Hello"}))
```

### Receiving Messages

```qi
;; Receive message (blocking)
(def msg (ws/receive conn))

;; Process based on message type
(match (get msg :type)
  "message" -> (println (get msg :data))
  "close" -> (println "Connection closed")
  "error" -> (println (get msg :error)))
```

### Closing Connection

```qi
;; Close connection
(ws/close conn)
```

## 18.2 Message Types

`ws/receive` returns a message map of one of the following types:

### Text Message

```qi
{:type "message" :data "received text"}
```

### Binary Message

```qi
{:type "binary" :data "base64 encoded data"}
```

Binary data is returned as a base64-encoded string.

### Close Message

```qi
{:type "close" :code 1000 :reason "Normal closure"}
```

Received when the server closes the connection.

### Error

```qi
{:type "error" :error "error message"}
```

Received when a communication error occurs.

## 18.3 Practical Patterns

### Chat Client

```qi
(defn chat-client [url username]
  (let [conn (ws/connect url)]
    (if (error? conn)
      (println f"Connection error: {conn}")
      (do
        ;; Send join message
        (ws/send conn (json/stringify {:type "join" :user username}))

        ;; Message receive loop
        (loop []
          (let [msg (ws/receive conn)]
            (match (get msg :type)
              "message" ->
                (do
                  (let [data (json/parse (get msg :data))]
                    (println f"{(get data \"user\")}: {(get data \"msg\")}"))
                  (recur))
              "close" -> (println "Connection disconnected")
              "error" -> (println f"Error: {(get msg :error)}")
              _ -> (recur))))

        (ws/close conn)))))

;; Usage example
(chat-client "ws://localhost:8080/chat" "Alice")
```

### JSON Message Send/Receive

```qi
;; JSON send helper
(defn send-json [conn data]
  (ws/send conn (json/stringify data)))

;; JSON receive helper
(defn receive-json [conn]
  (let [response (ws/receive conn)]
    (match (get response :type)
      "message" -> {:ok (json/parse (get response :data))}
      _ -> {:error response})))

;; Usage example
(def conn (ws/connect "ws://localhost:8080/api"))
(send-json conn {:action "subscribe" :channel "news"})
(def msg (receive-json conn))
```

### Error Handling

```qi
(defn safe-ws-connect [url max-retries]
  (loop [retries 0]
    (let [conn (try (ws/connect url))]
      (if (error? conn)
        (if (< retries max-retries)
          (do
            (println f"Connection failed. Retry {(+ retries 1)}/{max-retries}")
            (recur (+ retries 1)))
          (do
            (println "Reached maximum retry count")
            nil))
        conn))))

;; Usage example
(def conn (safe-ws-connect "ws://localhost:8080" 3))
```

### Receive with Timeout

WebSocket is blocking by default, but timeout can be implemented using goroutines and channels:

```qi
(defn ws-receive-timeout [conn timeout-ms]
  (let [ch (go/chan)
        timeout-ch (go/timeout timeout-ms)]
    ;; Execute receive in goroutine
    (go
      (let [msg (ws/receive conn)]
        (go/send ch msg)))
    ;; Wait for timeout or receive
    (go/select
      ch -> (go/recv ch)
      timeout-ch -> {:error "timeout"})))
```

## 18.4 Connection Management

### Managing Multiple Connections

Connection IDs are integers, so multiple connections can be managed with a map:

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
    (println f"Connection {name} not found")))

;; Usage example
(add-connection :server1 "ws://server1.com/ws")
(add-connection :server2 "ws://server2.com/ws")
(send-to :server1 "Hello from server1")
```

## 18.5 Server-Side Implementation

Currently, Qi only provides WebSocket client functionality. Server-side WebSocket functionality is planned for future versions.

## 18.6 Function Reference

### ws/connect

```qi
(ws/connect url) ;=> connection-id
```

Connects to a WebSocket server.

- **Arguments**: `url` - WebSocket URL (ws:// or wss://)
- **Return Value**: Connection ID (Integer)
- **Error**: Error string on connection failure

**Example**:
```qi
(def conn (ws/connect "wss://echo.websocket.org"))
```

### ws/send

```qi
(ws/send conn-id message) ;=> nil
```

Sends a message via WebSocket.

- **Arguments**:
  - `conn-id` - Connection ID
  - `message` - Message to send (string)
- **Return Value**: nil
- **Error**: Throws error on send failure

**Example**:
```qi
(ws/send conn "Hello, WebSocket!")
```

### ws/receive

```qi
(ws/receive conn-id) ;=> message-map
```

Receives a message from WebSocket. Blocks until a message arrives.

- **Arguments**: `conn-id` - Connection ID
- **Return Value**: Message map (`:type`, `:data`, `:code`, `:reason`, `:error`)
- **Error**: Throws error on receive failure

**Example**:
```qi
(def msg (ws/receive conn))
(println (get msg :type)) ;=> "message"
```

### ws/close

```qi
(ws/close conn-id) ;=> nil
```

Closes the WebSocket connection.

- **Arguments**: `conn-id` - Connection ID
- **Return Value**: nil
- **Error**: Throws error on close failure

**Example**:
```qi
(ws/close conn)
```

## 18.7 Best Practices

### 1. Error Handling

WebSocket connections can become unstable, so implement proper error handling:

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

### 2. Reconnection Logic

Implement automatic reconnection when the connection is lost due to network issues:

```qi
(defn auto-reconnect [url]
  (loop [delay 1000]
    (let [conn (try (ws/connect url))]
      (if (error? conn)
        (do
          (println f"Retrying connection in {delay}ms...")
          (go/sleep delay)
          (recur (min (* delay 2) 30000))) ;; exponential backoff
        conn))))
```

### 3. Resource Cleanup

Always close connections after use:

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

## 18.8 Security

### TLS/SSL

When handling sensitive information, always use `wss://` (WebSocket Secure):

```qi
;; ✅ Good example
(ws/connect "wss://api.example.com/secure")

;; ❌ Bad example (when handling sensitive information)
(ws/connect "ws://api.example.com/secure")
```

### Authentication

When including authentication tokens during WebSocket connection, send them via URL query parameters or the first message:

```qi
;; Authentication via URL query parameters
(def conn (ws/connect f"wss://api.example.com/ws?token={token}"))

;; Authentication via first message
(def conn (ws/connect "wss://api.example.com/ws"))
(ws/send conn (json/stringify {:type "auth" :token token}))
```

## 18.9 Performance

### Automatic Ping/Pong Handling

Qi's WebSocket implementation automatically handles Ping/Pong frames. Users do not need to take special action.

### Binary Data

Binary messages are returned base64-encoded. When handling large amounts of binary data, be aware of encoding/decoding overhead.

## Summary

Qi's WebSocket functionality makes real-time communication easy to implement:

- `ws/connect` - Establish connection
- `ws/send` - Send message
- `ws/receive` - Receive message (blocking)
- `ws/close` - Close connection

By properly implementing error handling and reconnection logic, you can build stable real-time applications.
