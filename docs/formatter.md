# Formatter Implementation Notes

Qi のフォーマッタは `src/formatter` 配下で実装されており、以下のモジュールが構文変更と密接に関係しています。

- `src/formatter/tokenizer.rs`: コメントや空白などトリビアを保持する専用トークナイザ
- `src/formatter/doc.rs`: トリビア付きの S 式ツリーを構築するパーサ
- `src/parser.rs`: 実行系で使用する構文解析器（言語仕様の単一の真実）

**構文を追加・変更する場合のチェックリスト**

1. `src/parser.rs` で構文ルールを更新する
2. `src/formatter/tokenizer.rs` で新しいトークンや挙動を追加する
3. `src/formatter/doc.rs` でドキュメントツリーの組み立てロジックを更新する

実装を同期しておかないと `qi fmt` の結果が壊れる可能性があります。フォーマッタとパーサを同時に更新するようにしてください。

- `src/formatter/layout.rs`: Docツリーから最終レイアウトを生成するエンジン（本資料の後半参照）

## レイアウトエンジンの構造

1. `tokenizer::tokenize` でトリビア付きトークン列を生成
2. `doc::parse_tokens` で Doc ツリーを構築
3. `layout::render_module` で Doc ツリーをスタイルガイドに沿った文字列へ変換

フォーマット出力に影響する仕様追加・変更がある場合は、上記 1～3 の全レイヤーを確認してください。

## `.qi-format.edn`

フォーマッタはカレントディレクトリにある `.qi-format.edn` を読み取り、数値設定を上書きします。現在サポートしているキーは次の通りです。

| キー | 説明 |
|------|------|
| `:indent-width` | インデント幅（デフォルト 2） |
| `:blank-lines-between-defs` | トップレベル定義の間に入れる空行数（デフォルト 1） |
| `:max-line-length` | 1 行の最大長（デフォルト 100） |

ファイルは単純な EDN マップで記述してください。例：

```clojure
{:indent-width 4
 :blank-lines-between-defs 2}
```

未対応キーや解析に失敗した場合は設定が無視され、デフォルト値にフォールバックします。

CLI は `qi fmt <path>` のほか、`qi fmt -` で標準入力からコードを受け取り、結果を標準出力へ返せます。`--check` の有無にかかわらず、stdin モードでは出力内容をファイルに書き戻しません。
