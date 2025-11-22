use super::*;
use crate::builtins::db::types::*;

pub trait DbDriver: Send + Sync {
    /// 接続URLからデータベースに接続
    fn connect(&self, url: &str, opts: &ConnectionOptions) -> DbResult<Arc<dyn DbConnection>>;

    /// ドライバー名を取得
    fn name(&self) -> &str;
}

/// データベース接続の統一インターフェース
pub trait DbConnection: Send + Sync {
    /// SQLクエリを実行して結果を取得
    fn query(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Rows>;

    /// SQLクエリを実行して最初の1行のみ取得
    fn query_one(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Option<Row>> {
        let rows = self.query(sql, params, opts)?;
        Ok(rows.into_iter().next())
    }

    /// SQL文を実行して影響を受けた行数を返す
    fn exec(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<i64>;

    /// トランザクションを開始
    fn begin(&self, opts: &TransactionOptions) -> DbResult<Arc<dyn DbTransaction>>;

    /// 接続を閉じる
    fn close(&self) -> DbResult<()>;

    /// サニタイズ: 値をエスケープ（シングルクォート対応）
    fn sanitize(&self, value: &str) -> String;

    /// サニタイズ: 識別子をエスケープ（テーブル名、カラム名）
    fn sanitize_identifier(&self, name: &str) -> String;

    /// LIKE句のパターンをエスケープ
    fn escape_like(&self, pattern: &str) -> String;

    /// ドライバー名を取得
    fn driver_name(&self) -> &str;

    // Phase 2: メタデータAPI
    /// テーブル一覧を取得
    fn tables(&self) -> DbResult<Vec<String>>;

    /// テーブルのカラム情報を取得
    fn columns(&self, table: &str) -> DbResult<Vec<ColumnInfo>>;

    /// テーブルのインデックス一覧を取得
    fn indexes(&self, table: &str) -> DbResult<Vec<IndexInfo>>;

    /// テーブルの外部キー一覧を取得
    fn foreign_keys(&self, table: &str) -> DbResult<Vec<ForeignKeyInfo>>;

    /// ストアドプロシージャ/ファンクションを呼び出す
    fn call(&self, name: &str, params: &[Value]) -> DbResult<CallResult>;

    /// 機能をサポートしているか確認
    fn supports(&self, feature: &str) -> bool;

    /// ドライバー情報を取得
    fn driver_info(&self) -> DbResult<DriverInfo>;

    /// クエリのメタデータを取得（実行せずにカラム情報を取得）
    fn query_info(&self, sql: &str) -> DbResult<QueryInfo>;
}

/// データベーストランザクションの統一インターフェース
pub trait DbTransaction: Send + Sync {
    /// SQLクエリを実行して結果を取得
    fn query(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Rows>;

    /// SQLクエリを実行して最初の1行のみ取得
    fn query_one(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<Option<Row>> {
        let rows = self.query(sql, params, opts)?;
        Ok(rows.into_iter().next())
    }

    /// SQL文を実行して影響を受けた行数を返す
    fn exec(&self, sql: &str, params: &[Value], opts: &QueryOptions) -> DbResult<i64>;

    /// ストアドプロシージャ/ファンクションを呼び出す
    fn call(&self, name: &str, params: &[Value]) -> DbResult<CallResult>;

    /// トランザクションをコミット
    fn commit(self: Arc<Self>) -> DbResult<()>;

    /// トランザクションをロールバック
    fn rollback(self: Arc<Self>) -> DbResult<()>;
}
