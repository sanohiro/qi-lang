# Qi Standard Library

Qiの標準ライブラリコレクションです。

## 利用可能なライブラリ

### OpenAPI (`std/lib/openapi.qi`)

REST APIのためのOpenAPI 3.0仕様生成ライブラリ。

**機能**:
- APIエンドポイントの宣言的定義
- OpenAPI 3.0仕様の自動生成
- Swagger JSONエンドポイントの提供

**使い方**:
```qi
;; シンプルなインポート（推奨）
(use "openapi" :as openapi)

;; または、フルパス指定
(use "std/lib/openapi" :as openapi)

;; APIエンドポイントを定義
(openapi/defapi :post "/api/users"
  {:summary "ユーザー登録"
   :requestBody {...}
   :responses {201 {:description "Created"}}}
  api-create-user
  (let [body (json/parse (get req :body))]
    ...))

;; Swaggerエンドポイントを統合
(def router
  (openapi/with-swagger api-router
    {:title "My API" :version "1.0.0"}))

;; サーバー起動
(server/serve router {:port 3000})
```

詳細: [openapi.md](openapi.md)

## ライブラリの追加

新しいライブラリを追加する場合は、以下の構成に従ってください：

```
std/lib/
├── mylib.qi          # ライブラリ本体
├── mylib.md          # ドキュメント
└── README.md         # このファイル
```

### ライブラリの要件

1. **モジュール宣言**: 明確なモジュール名を定義
2. **export宣言**: 公開する関数を明示
3. **ドキュメント**: 使用例とAPIリファレンスを含む
4. **テスト**: 動作確認用のサンプルコード
