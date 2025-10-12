
# 🧠 Qi言語 モジュール設計・ビルド方針まとめ

## 1. 開発コンセプト

**Qi - A Lisp that flows**  
> 「データは流れ、プログラムは流れを設計する」

Qiの設計は **Flow-Oriented Programming** を中心に据え、  
すべてを“データの流れ”として扱うことを目的とする。

請負原則：
- 「できるだけ小さく、一貫して動く」
- 「Flowの入口（入力）と出口（出力）を統一した抽象で扱う」
- 「マルチスレッド・並列・非同期をネイティブに扱う」

---

## 2. システム構造の基本思想

QiはRust製の高性能インタプリタ。  
アーキテクチャは下記3層構造：

```
Layer 3: async/await（高レベル）
Layer 2: Pipeline / Stream（中レベル）
Layer 1: go/chan（並行処理基盤）
```

これに「Lazy Init + feature構成」で、  
起動軽量性・モジュール拡張性・配布容易性を全て両立する。

---

## 3. モジュール実装方針

| 種別 | 実装形態 | 初期化方針 | 備考 |
|-------|-------------|-------------|------|
| **Flow核 (core, pipeline, stream)** | 常時組み込み | 即時初期化 | 言語の核（Flowを構築） |
| **I/O・HTTP 系 (io, path, http, server, json)** | 常時組み込み | Lazy Init | 利用頻度高、依存軽い |
| **DB 系 (sqlite, postgres, mysql, duckdb, turso, odbc)** | 条件付きコンパイル + Lazy Init | 利用時初期化 | 機能別featureで制御 |
| **重依存 (zip, tls, compression, odbc)** | optional feature | Lazy InitまたはDLL | サイズ削減・柔軟化 |
| **ユーティリティ (fn, set, math, csv, dbg)** | 常時組み込み | 即時/軽量初期化 | 依存軽く常用される |

---

## 4. データベース統合設計

### 目的
- 複数DBを**統一インターフェース**で扱う（Goの`database/sql`思想を継承）
- DBごとの初期化コストをLazy Initで吸収
- バイナリ配布時の軽量化とfeatureビルド対応の両立

### 設計構造
```rust
// db_registry.rs
#[cfg(feature = "postgres")] mod db_postgres;
#[cfg(feature = "mysql")] mod db_mysql;
// ...

pub fn connect(url: &str) -> Result<Connection> {
    if url.starts_with("postgres://") {
        #[cfg(feature = "postgres")]
        { return db_postgres::connect_lazy(url); }
        #[cfg(not(feature = "postgres"))]
        { return Err("PostgreSQL driver not enabled".into()); }
    }
    // 同様に mysql, duckdb, turso...
    Err("Unsupported driver".into())
}
```

### Rust実装概要
- 全DBドライバは `OnceCell` / `Lazy` により **初回利用時にのみ初期化**
- featureで有効化された分のみコンパイルし、未使用モジュールは除外
- Qi側では `(db/connect "postgres://...")` のように**統一文法で利用可能**
- 例：
  ```lisp
  (db/connect "postgres://user:pass@localhost/db"
   |> (db/query "SELECT * FROM users")
   |> (map :email)
   |> unique)
  ```

### Go言語との対応関係

| Go | Qi |
|----|----|
| `database/sql` 抽象層 | `db/core` + `db_registry` |
| `sql.Register` | `db_registry::register_driver()` |
| 各ドライバ (`lib/pq`, `mysql_async`) | feature + Lazy Initモジュール |
| `sql.Open(...)` | `(db/connect "...")` |

→ QiはGoの「インターフェースだけ標準」思想を**ビルドオプションとLazy Initで動的に再現**。

---

## 5. Lazy Initの設計

### コンセプト
> 「コードは持っているが、必要になるまで起動しない。」

Lazy Initを導入することで：
- 起動時の常駐負荷は0
- 未使用のモジュールはメモリ割当なし
- 初回利用時にのみ依存を起動・プール作成

### 実装例
```rust
use once_cell::sync::OnceCell;
static PG_RUNTIME: OnceCell<PgDriver> = OnceCell::new();

pub fn connect_lazy(url: &str) -> Result<Connection> {
    let driver = PG_RUNTIME.get_or_init(|| PgDriver::new());
    driver.connect(url)
}
```

---

## 6. Cargo features戦略

### Cargo.tomlの構成例
```toml
[features]
default = [
  "core", "io", "json", "server", "stream", "sqlite"
]

# Heavy DB modules
postgres = []
mysql = []
duckdb = []
turso = []
odbc = []

# Heavy subsystems
zip = []
server-tls = []
http-compression = []

# Extra utilities
math = []
csv = []
fn = []
set = []
log = []
```

### ビルド例
| 用途 | コマンド | 内容 |
|------|-----------|------|
| 軽量配布版 | `cargo build --release` | デフォルト機能のみ |
| フルDB版 | `cargo build --release --features "postgres mysql duckdb turso"` | 全DB統合 |
| ODBC付き実験版 | `cargo build --release --features "odbc"` | C依存有効 |
| テスト・開発版 | `cargo build --all-features` | 全モジュール有効 |

---

## 7. サーバーモジュール方針

### 主な内容
- `server/serve`, `server/router`, `server/json` 等をFlow出口として統合
- スレッド・ソケットは Lazy Init により `server/serve` 読び込み時のみ初期化
- `go/chan` と直接連携（リクエスト並列実行）
- JSON / HTTP と統合されたFlow-orientedデザイン  

### 理由
- Qiの哲学上、**サーバーはI/Oエンドポイントの一形態**
- lazy化すれば起動負担ゼロで統合可能
- 重いTLS圧縮は `server-tls`, `server-full` feature で切り替え

---

## 8. 配布モード（推奨設計）

| バージョン | 機能セット | 備考 |
|-------------|-------------|------|
| **Qi Core** | default features（sqlite, http, json, server, stream） | 最小構成・公式配布 |
| **Qi Full** | full-database (`postgres mysql duckdb turso`) | DB統合版 |
| **Qi Dev** | all-features | 内部/開発者用 |
| **Qi Lite** | no-db, no-server | スクリプト／埋め込み用途 |

---

## 9. 将来の拡張・ロード戦略

| 分類 | 方針 |
|------|------|
| ODBC・Mongo/Redis | 外部プラグイン化（FFI/DLL方式） |
| Tensor / AI 系 | 別バイナリ/外部モジュール想定 |
| WebSocket, streaming HTTP | 同serverモジュール内で追加予定 |
| Plugin loader | `libloading`ベース動的ロード対応予定 |
| Docシステム | `defn`のドキュメント文字列を後利用予定 |

---

## 10. 結論

| 項目 | 方針 |
|------|------|
| **DB** | 標準統合・feature選択 + Lazy Init |
| **サーバー** | 組込み + Lazy Init |
| **HTTP/JSON/Stream** | 常時標準 |
| **ODBC/TLS/ZIP** | heavy feature |
| **配布方法** | バイナリ配布を基本、ソースビルドでカスタマイズ |
| **全体メリット** | 安全・統一・自己完結・即実行 |

> Qiの哲学に最も沿う構成は：
> **「すべてがFlowに統合された単一言語環境」＋「必要時にのみ起動するLazy System」＋「ビルド時のCargo featuresで拡張自由」**

---

## ✳️ まとめ図：Qiモジュールレイヤ

```
┌────────────────────────────────────┐
│         APPLICATION / FLOW          │
│  (pipelines, match, async/await)    │
├────────────────────────────────────┤
│     CORE MODULES (常時組込み)       │
│  core, io, path, http, server, json │
├────────────────────────────────────┤
│   FEATURE MODULES (条件コンパイル)  │
│  postgres, mysql, duckdb, turso,    │
│  zip, odbc, tls                     │
├────────────────────────────────────┤
│  FLOW ENGINE (go/chan + stream)     │
│  lazy init / oncecell runtime       │
└────────────────────────────────────┘
```

---

以上が、  
💾 **「Qi言語のモジュール設計とビルド戦略の決定版まとめ」** です。
