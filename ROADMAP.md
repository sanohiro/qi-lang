# Qi言語ロードマップ

**未実装機能と将来の計画**

このドキュメントでは、Qi言語の未実装機能と将来の計画をまとめています。実装済み機能については `docs/spec/` ディレクトリを参照してください。

---

## 📋 優先度高（次期実装予定）

### APIサーバー・アプリケーション開発機能

#### 1. ファイル監視 📁

```qi
;; ファイル・ディレクトリ監視
(def watcher
  (fs/watch "src"
    {:on-create (fn [path] (log f"Created: {path}"))
     :on-modify (fn [path] (log f"Modified: {path}"))
     :on-delete (fn [path] (log f"Deleted: {path}"))
     :on-rename (fn [old new] (log f"Renamed: {old} -> {new}"))}))

;; ホットリロード機能
(def server
  (server/serve app
    {:port 3000
     :hot-reload true
     :watch-dir "src"}))
;; ファイル変更時に自動的にリロード
```

#### 2. ログ高度機能 📊

```qi
;; ログ出力先指定
(log/configure
  {:level :info
   :outputs [{:type :file :path "app.log"}
             {:type :stdout :format :json}
             {:type :syslog :host "localhost"}]})

;; ログローテーション
(log/configure
  {:outputs [{:type :file
              :path "app.log"
              :rotation :daily  ;; or :size
              :max-files 7
              :compress true}]})

;; 非同期ログ出力（パフォーマンス向上）
(log/configure {:async true :buffer-size 1000})
```

#### 3. メトリクス・モニタリング 📈

```qi
;; カウンター
(def requests-counter (metrics/counter "http_requests_total"))
(metrics/inc requests-counter)

;; ゲージ
(def memory-gauge (metrics/gauge "memory_usage_bytes"))
(metrics/set memory-gauge 1024000)

;; ヒストグラム
(def duration-histogram (metrics/histogram "http_request_duration_seconds"))
(metrics/observe duration-histogram 0.125)

;; Prometheus形式エクスポート
(server/serve (metrics/handler) {:port 9090})
;; => http://localhost:9090/metrics

;; APM連携
(metrics/configure {:apm {:provider :datadog :api-key "..."}})
```

---

## 📌 優先度中（将来検討）

### パイプライン拡張

#### flow DSL - 分岐・合流を含む複雑な流れ

```qi
;; 複雑なデータフローを構造化
(flow data
  -> parse
  -> (branch
       [valid? -> process]
       [invalid? -> log-error])
  -> merge
  -> save)

;; 実用例: データ処理パイプライン
(flow raw-data
  -> clean
  -> (split
       [:numeric -> (branch
                      [outlier? -> remove-outlier]
                      [normal? -> normalize])]
       [:categorical -> encode])
  -> merge
  -> model/predict)
```

### パターンマッチング拡張

#### => 変換パターン - マッチ時にデータを変換

```qi
;; 束縛と同時に変換関数を適用（パイプライン的）
(match data
  {:price p => parse-float} -> (calc-tax p)
  {:name n => lower} -> (log n)
  {:created-at t => parse-date} -> (format t))

;; 複数の変換をつなげる
(match input
  {:raw r => trim => lower => (split " ")} -> (process-words r))

;; 実用例: APIレスポンス処理
(match (http/get "/api/user")
  {:body b => json/parse} -> (extract-user b)
  {:status s => str} when (= s "404") -> nil
  _ -> (error "unexpected response"))
```

### 正規表現（regex）拡張

**Phase 2以降の機能**:

```qi
;; 名前付きキャプチャ
(regex/matches "(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})" "2024-01-15")
;; => {:ok {:year "2024" :month "01" :day "15"}}

;; 複数マッチの詳細情報
(regex/find-all "\\d+" "abc123def456ghi")
;; => [{:match "123" :start 3 :end 6}
;;     {:match "456" :start 9 :end 12}]
```

### 時刻処理拡張

**Phase 4: タイムゾーン対応**:

```qi
;; タイムゾーン変換
(time/to-timezone (time/now) "America/New_York")
;; => "2024-01-15T09:30:00-05:00"

;; タイムゾーン情報付き日時
(time/parse "2024-01-15T14:30:00+09:00")
;; => {:ok #inst "2024-01-15T05:30:00Z"}
```

---

## 📍 優先度低（長期計画）

### JITコンパイル

**現在の実行速度**: 中速〜高速（インタープリタ方式）

**将来の計画**:
- JITコンパイラ導入による高速化
- ホットパス最適化
- インライン展開

### 名前空間システム（Phase 6以降）

**現状**: グローバル名前空間のみ

**将来検討**:
```qi
;; 案1: Clojure風
(ns myapp.core)
(def map {...})  ;; myapp.core/map

(myapp.core/map ...)  ;; 自分のmap
(core/map ...)        ;; 組み込みmap

;; 案2: モジュールシステム拡張
(module myapp
  (def map {...}))

(myapp/map ...)
```

**優先度**: 低（設計思想「シンプル」に反するため、必要になったら検討）

---

## 📚 関連ドキュメント

- **[docs/spec/](docs/spec/)** - 実装済み機能の完全仕様
- **[README.md](README.md)** - プロジェクト概要
- **[CLAUDE.md](CLAUDE.md)** - 開発者向けガイド
- **[docs/style-guide.md](docs/style-guide.md)** - コーディングスタイルガイド

---

## 📝 ドキュメント更新履歴

- 2025-01-XX: 実装済み機能を削除、未実装機能のみに整理
- 2025-01-XX: 初版作成（SPEC.mdから未実装機能を抽出）
