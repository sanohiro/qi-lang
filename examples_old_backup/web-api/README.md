# Web API Examples

Phase 4.5で実装されたWeb開発機能の実用例です。

## 機能

- **Railway Pipeline (`|>?`)**: エラーハンドリングを流れの中に組み込む
- **JSON処理**: `json/parse`, `json/stringify`, `json/pretty`
- **HTTP クライアント**: `http/get`, `http/post`など全メソッド対応
- **デバッグ**: `inspect`, `time`
- **コレクション拡張**: `find`, `every?`, `some?`, `update-keys`, `update-vals`, `zipmap`

## サンプルファイル

### 1. `github_user.qi`
GitHub APIからユーザー情報を取得し表示します。

```bash
cargo run -- -l examples/web-api/github_user.qi
```

**内容**:
- HTTP GET リクエスト
- JSONパース
- エラーハンドリング（Railway Pipeline）
- パターンマッチング

### 2. `api_pipeline.qi`
複数のAPIリクエストを並列処理し、データを変換します。

```bash
cargo run -- -l examples/web-api/api_pipeline.qi
```

**内容**:
- 並列パイプライン (`||>`)
- Railway Pipeline (`|>?`)
- データ変換（`update-keys`, `filter`）
- デバッグ出力（`inspect`）

### 3. `json_transform.qi`
JSON データを段階的に変換するパイプラインの例。

```bash
cargo run -- -l examples/web-api/json_transform.qi
```

**内容**:
- JSON パース/生成
- 複雑なデータ変換
- 中間ステップの可視化
- エラーハンドリング

## 学習ポイント

### Railway Pipeline パターン
```lisp
(data
 |> http/get              ;; {:ok {...}} または {:error ...}
 |>? (fn [resp] ...)      ;; :okなら実行、:errorならスキップ
 |>? json/parse
 |>? validate
 |>? save)
```

### データ変換パイプライン
```lisp
(users
 |> (filter active?)
 |> (map transform)
 |> (update-vals sanitize)
 |> json/stringify)
```

### 並列処理 + エラーハンドリング
```lisp
(urls
 ||> http/get             ;; 並列リクエスト
 |> (map extract-data)
 |>? aggregate            ;; Result型の処理
 |>? save-results)
```

## 設計哲学

1. **データは流れる**: すべての処理がパイプラインで表現される
2. **エラーは流れる**: `|>?`でエラーが自動的に伝播
3. **段階的変換**: 小さな変換を組み合わせて大きな処理を作る
4. **観察可能**: `inspect`と`time`でデータフローを可視化

## 次のステップ

これらの例を参考に、独自のWebアプリケーションを作ってみましょう：

- REST APIクライアント
- データ収集・分析スクリプト
- Webスクレイパー
- マイクロサービス
