# 基礎的なモダン文字列ライブラリ — 実装状況＋説明（Qi SPEC準拠）

## 1. 型判定・基本情報
- ✅ `string?` – 値が文字列かどうかを判定。あらゆる文字列関数の前提となる。  
- ✅ `len` – 文字列長（Unicodeやバイト数ではない通常カウント）。配列やリストでも使える。  
- ✅ `empty?` – 空文字列判定。条件分岐やバリデーションで頻用。  
- 🚧 `chars-count` – Unicode面の堅牢性向上。絵文字や結合文字など「見た目の文字数」を数える。  
- 🚧 `bytes-count` – バイト単位の長さを取得。保存時や通信時の容量制御に必須。

---

## 2. 連結・繰り返し
- ✅ `str` – 複数文字列の連結。パイプライン終端で多用。  
- 🚧 `repeat` – 文字列の複製。例: (repeat "-" 80) で区切り線生成。CLI/UIなどで有用。

---

## 3. 変換
- ✅ `upper` – 大文字変換。UIやデータフォーマット統一に使用。  
- ✅ `lower` – 小文字変換。比較の前処理、slug生成、正規化に必須。  
- 🚧 `capitalize` – 先頭文字のみ大文字にする。人名やタイトル整形。  
- ✅ `trim` – 前後の空白除去。入力値クリーニングで必須。  
- 🚧 `trim-left` – 左側だけ空白除去。インデント調整など。  
- 🚧 `trim-right` – 右側だけ空白除去。フォーマット調整など。

---

## 4. 検索
- 🚧 `contains?` – 部分一致判定。フィルタやバリデーションで頻用。  
- 🚧 `starts-with?` – 接頭辞判定。URLやファイル拡張子チェックなど。  
- 🚧 `ends-with?` – 接尾辞判定。拡張子やプロトコル判定に必須。  
- 🚧 `index-of` – 部分文字列の開始位置。  
- 🚧 `last-index-of` – 最後に見つかった位置。ログ解析やタグ抽出などで有用。

---

## 5. 分割・結合
- ✅ `split` – 区切り文字で分割。CSVや行解析などに基本。  
- 🚧 `lines` – 改行単位で分割。テキスト処理やスクレイピングで重要。  
- 🚧 `words` – 空白単位で分割。NLPやキーワード抽出に必須。  
- ✅ `join` – 分割結果の再結合。パイプライン終端でフォーマット整形に使用。

---

## 6. 置換
- 🚧 `replace` – 全ての一致部分を置換。フォーマット変換やクレンジングに必須。  
- 🚧 `replace-first` – 最初の一致だけ置換。制御文字変換など部分的変更に便利。

---

## 7. 部分取得
- 🚧 `slice` – インデックス範囲で抽出。サブ文字列取得の基本。  
- 🚧 `take-str` – 先頭から指定数取得。ログプレビューや短縮表示用。  
- 🚧 `drop-str` – 先頭から指定数削除。プレフィックス除去や省略表示に使用。  
- 🚧 `sub-before` – 指定区切り文字より前の部分を取得。キーやパスの抽出に有用（SPEC未記載）。  
- 🚧 `sub-after` – 指定区切り文字より後の部分。拡張子やクエリ抽出に便利（SPEC未記載）。

---

## 8. フォーマット基礎
- 🚧 `pad-left` – 左側に指定文字を詰める。整列やコード生成に使用。  
- 🚧 `pad-right` – 右側に詰める。表やログ整形で便利。  
- 🚧 `indent` – インデント追加。コード生成、Markdown生成など。  
- 🚧 `wrap` – 指定幅で改行。CLI出力整形やメール文生成に有用。

---

## 📊 現状総評
- **現行SPECで完全実装済み**  
  → `string?`, `len`, `empty?`, `str`, `upper`, `lower`, `trim`, `split`, `join`  
- **SPECに名前のみあり・未実装**  
  → capitalize, trim-left/right, contains?, starts/ends-with?, index-of, last-index-of, lines, words, replace, replace-first, slice, take-str, drop-str, pad-left/right, indent, wrap  
- **SPEC未記載だが追加推奨**  
  → repeat, chars-count, bytes-count, sub-before, sub-after

---

## 💡 改善の優先度（Web／LLM利用想定）
1. 検索系の充実（contains?, starts/ends-with?, index-ofなど）  
2. 部分取得系の追加（slice, sub-before/sub-after）  
3. テキストフォーマット系の追加（pad、indent、wrap）  
4. Unicode対応（chars-count/bytes-count）  
5. 繰り返し（repeat）