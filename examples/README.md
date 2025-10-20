# Qi Examples - Qiの例

**Qiの真髄を学ぶ16の実践的な例**

このディレクトリには、Qi言語の強力な機能を示す実践的な例が含まれています。
基本から応用まで、順を追って学習できるように構成されています。

---

## 📚 目次

### 🔰 基本編（Basic）
初心者向け - Qiの基本構文とデータ構造を学ぶ

| ファイル | 内容 | 所要時間 |
|---------|------|---------|
| [01-hello.qi](01-hello.qi) | Hello World、基本構文、f-string | 10分 |
| [02-functions.qi](02-functions.qi) | 関数定義、クロージャ、高階関数 | 15分 |
| [03-data-structures.qi](03-data-structures.qi) | ベクター、リスト、マップ、セット | 15分 |

### ⭐ ウリ編（Showcase - Qiの真骨頂）
Qiの3大ウリを体験する

| ファイル | 内容 | 所要時間 |
|---------|------|---------|
| [04-pipelines.qi](04-pipelines.qi) | **パイプライン演算子** - Flow-Oriented Programming | 20分 |
| [05-pattern-matching.qi](05-pattern-matching.qi) | **パターンマッチング** - データ構造の分解 | 20分 |
| [06-error-handling.qi](06-error-handling.qi) | **Railway Pipeline** - エラーを優雅に扱う | 20分 |
| [07-concurrency.qi](07-concurrency.qi) | **並行・並列処理** - Qiのキモ！ | 25分 |

### 🔧 Unix統合編（Unix Integration - 超強力！）
UnixコマンドとQiの融合

| ファイル | 内容 | 所要時間 |
|---------|------|---------|
| [08-command-pipelines.qi](08-command-pipelines.qi) | **Unixコマンドパイプライン** - cmd/exec | 20分 |
| [09-log-analysis.qi](09-log-analysis.qi) | ログ解析パイプライン | 15分 |
| [10-file-processing.qi](10-file-processing.qi) | ファイル処理パイプライン | 15分 |

### 🌐 Web編（Web - 必須！）
Webアプリケーション構築

| ファイル | 内容 | 所要時間 |
|---------|------|---------|
| [11-http-client.qi](11-http-client.qi) | HTTPクライアント - REST API呼び出し | 15分 |
| [12-web-server.qi](12-web-server.qi) | **Webサーバー** - ルーティング、ミドルウェア | 25分 |
| [13-rest-api.qi](13-rest-api.qi) | RESTful API - CRUD、バリデーション | 25分 |

### 💡 実践編（Real-world）
実践的なデータ処理

| ファイル | 内容 | 所要時間 |
|---------|------|---------|
| [14-json-transform.qi](14-json-transform.qi) | JSONデータ変換パイプライン | 20分 |
| [15-parallel-data.qi](15-parallel-data.qi) | 並列データ処理 | 20分 |
| [16-stdin-processing.qi](16-stdin-processing.qi) | **標準入力処理** - Unix パイプライン | 15分 |

---

## 🎯 Qiの3大ウリ（★）

### 1. **パイプライン演算子** - Flow-Oriented Programming

**「データは流れ、プログラムは流れを設計する」**

```qi
(data
 |> parse
 |> (filter valid?)
 |> (map transform)
 |> save)
```

- `|>` - 逐次パイプライン（左から右へデータが流れる）
- `||>` - **並列パイプライン**（自動並列化！）
- `|>?` - **Railway Pipeline**（エラー処理）
- `tap>` - 副作用タップ（デバッグに便利）
- `~>` - 非同期パイプライン

**例**: [04-pipelines.qi](04-pipelines.qi)

### 2. **並行・並列処理** - 簡単で強力

**並列、並行を簡単にできるのはQiのキモ**

```qi
;; 自動並列化
(urls ||> http/get)

;; goroutine風
(def result (data ~> transform))

;; Atom（スレッドセーフ）
(def counter (atom 0))
(swap! counter inc)
```

- `||>` - 並列パイプライン（自動並列化）
- `pmap`, `pfilter`, `preduce` - 並列コレクション操作
- Atom - スレッドセーフな状態管理
- goroutine風 - `go/chan`, `go/send!`, `go/recv!`

**例**: [07-concurrency.qi](07-concurrency.qi), [15-parallel-data.qi](15-parallel-data.qi)

### 3. **パターンマッチング** - データフローを制御

```qi
(match response
  {:status 200 :body body} -> (process-body body)
  {:status 404} -> nil
  {:error e} -> (log-error e))
```

- データ構造の分解
- ガード条件（`when`）
- orパターン（`|`）
- Railway Oriented Programming

**例**: [05-pattern-matching.qi](05-pattern-matching.qi), [06-error-handling.qi](06-error-handling.qi)

---

## 🚀 実行方法

### 個別のファイルを実行

```bash
# Hello World
qi examples/01-hello.qi

# パイプライン（Qiのウリ）
qi examples/04-pipelines.qi

# Webサーバー
qi examples/12-web-server.qi
```

### 全ての例を順番に実行

```bash
for f in examples/*.qi; do
  echo "=== Running $f ==="
  qi "$f"
  echo ""
done
```

---

## 📖 学習パス

### 初心者コース（1時間）

1. [01-hello.qi](01-hello.qi) - 基本構文
2. [02-functions.qi](02-functions.qi) - 関数
3. [04-pipelines.qi](04-pipelines.qi) - パイプライン（★）
4. [05-pattern-matching.qi](05-pattern-matching.qi) - パターンマッチング（★）

### 中級コース（2時間）

1. 初心者コース（上記）
2. [06-error-handling.qi](06-error-handling.qi) - Railway Pipeline（★）
3. [07-concurrency.qi](07-concurrency.qi) - 並行・並列処理（★）
4. [12-web-server.qi](12-web-server.qi) - Webサーバー（必須！）

### 上級コース（3時間）

1. 中級コース（上記）
2. [08-command-pipelines.qi](08-command-pipelines.qi) - Unixコマンド統合
3. [13-rest-api.qi](13-rest-api.qi) - REST API実装
4. [14-json-transform.qi](14-json-transform.qi) - JSONデータ処理
5. [15-parallel-data.qi](15-parallel-data.qi) - 並列データ処理

---

## 💻 実践的なユースケース

### ログ解析ツール

```qi
(io/read-file "/var/log/app.log")
|> (str/split "\n")
|> (filter (fn [line] (str/includes? line "ERROR")))
|> (map parse-log-line)
|> (group-by (fn [log] (get log :hour)))
|> (map (fn [[hour logs]] {:hour hour :count (len logs)}))
```

**例**: [09-log-analysis.qi](09-log-analysis.qi)

### REST API サーバー

```qi
(defn api-handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/users"] -> (server/json {:users (get-all-users)})
    ["POST" "/api/users"] -> (create-user req)
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 api-handler)
```

**例**: [12-web-server.qi](12-web-server.qi), [13-rest-api.qi](13-rest-api.qi)

### 並列データ処理

```qi
(large-dataset
 |> (go/pfilter valid?)    ; 並列フィルタ
 ||> transform             ; 並列変換
 |> (go/preduce + 0))      ; 並列集約
```

**例**: [15-parallel-data.qi](15-parallel-data.qi)

---

## 🔗 関連ドキュメント

- **[チュートリアル](../docs/tutorial/)** - 初心者向け6章構成チュートリアル
- **[言語仕様書](../docs/spec/)** - 完全な言語リファレンス
- **[クイックリファレンス](../docs/spec/QUICK-REFERENCE.md)** - 1ページでQiの全体像
- **[README](../README.md)** - プロジェクト概要

---

## 🎓 さらに学ぶ

### 特定の機能を深掘り

- **パイプライン**: [docs/spec/02-flow-pipes.md](../docs/spec/02-flow-pipes.md)
- **並行処理**: [docs/spec/03-concurrency.md](../docs/spec/03-concurrency.md)
- **パターンマッチング**: [docs/spec/04-match.md](../docs/spec/04-match.md)
- **HTTP**: [docs/spec/11-stdlib-http.md](../docs/spec/11-stdlib-http.md)
- **文字列操作**: [docs/spec/10-stdlib-string.md](../docs/spec/10-stdlib-string.md)

### コミュニティ

- GitHub: [https://github.com/yourusername/qi-lang](https://github.com/yourusername/qi-lang)
- Issues: バグ報告・機能要望
- Discussions: 質問・議論

---

## 📝 ライセンス

このexamplesディレクトリはQi言語プロジェクトの一部であり、同じライセンスに従います。

---

**Qiでコードを書くのは楽しい！さあ、始めましょう 🚀**
