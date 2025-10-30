# Qi CMS Server

モジュール分割したQi言語によるCMSサーバー実装例

## 特徴

- **モジュール分割設計**: models/handlers/middleware/utilsに分離
- **Railway Pipeline**: Flow指向のエラーハンドリング
- **JWT認証**: ユーザー認証・認可
- **SQLiteデータベース**: 軽量で設定不要
- **RESTful API**: 標準的なHTTP APIインターフェース

## ディレクトリ構造

```
cms/
├── main.qi              # メインエントリーポイント
├── db.qi                # データベース初期化
├── models/              # データモデル層
│   ├── user.qi          # ユーザーモデル
│   ├── post.qi          # 記事モデル
│   └── comment.qi       # コメントモデル
├── handlers/            # HTTPハンドラー層
│   ├── auth.qi          # 認証ハンドラー
│   ├── posts.qi         # 記事ハンドラー
│   └── comments.qi      # コメントハンドラー
├── middleware/          # ミドルウェア層
│   └── auth.qi          # 認証ミドルウェア
└── utils/               # ユーティリティ層
    ├── auth.qi          # JWT認証ユーティリティ
    ├── slug.qi          # スラッグ生成
    └── pagination.qi    # ページネーション
```

## 起動方法

```bash
# CMSサーバー起動
qi examples/cms/main.qi
```

サーバーは `http://127.0.0.1:3000` で起動します。

## APIエンドポイント

### 認証

**ユーザー登録**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123","role":"admin"}'
```

**ログイン**
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

レスポンス例:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {"id": 1, "username": "alice", "role": "admin"}
}
```

### 記事管理

**記事一覧取得**
```bash
# 全記事
curl http://localhost:3000/api/posts

# 公開済み記事のみ
curl http://localhost:3000/api/posts?status=published

# ページネーション
curl http://localhost:3000/api/posts?page=2&per_page=5
```

**記事作成（要認証）**
```bash
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"title":"My First Post","content":"Hello, Qi CMS!"}'
```

**記事詳細取得**
```bash
curl http://localhost:3000/api/posts/my-first-post
```

**記事更新（要認証）**
```bash
curl -X PUT http://localhost:3000/api/posts/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"title":"Updated Title","content":"Updated content","status":"published"}'
```

**記事削除（要認証）**
```bash
curl -X DELETE http://localhost:3000/api/posts/1 \
  -H "Authorization: Bearer <token>"
```

### コメント管理

**コメント作成**
```bash
curl -X POST http://localhost:3000/api/posts/1/comments \
  -H "Content-Type: application/json" \
  -d '{"author_name":"Bob","content":"Great post!"}'
```

## データベーススキーマ

### users テーブル
- `id`: INTEGER PRIMARY KEY
- `username`: TEXT UNIQUE
- `password_hash`: TEXT
- `role`: TEXT (default: 'user')
- `created_at`: DATETIME

### posts テーブル
- `id`: INTEGER PRIMARY KEY
- `title`: TEXT
- `slug`: TEXT UNIQUE
- `content`: TEXT
- `author_id`: INTEGER (FK)
- `status`: TEXT (default: 'draft')
- `featured_image`: TEXT
- `created_at`: DATETIME
- `updated_at`: DATETIME

### comments テーブル
- `id`: INTEGER PRIMARY KEY
- `post_id`: INTEGER (FK)
- `author_name`: TEXT
- `content`: TEXT
- `created_at`: DATETIME

## セキュリティ

- パスワードは`password/hash`でハッシュ化
- JWT認証でAPIエンドポイントを保護
- パラメータ化クエリでSQLインジェクション防止

## 拡張案

- [ ] 画像アップロード機能
- [ ] カテゴリ・タグ機能
- [ ] 全文検索
- [ ] Redis キャッシュ
- [ ] PostgreSQL/MySQL対応
- [ ] WebSocket通知
