# Qi言語概要

**Qi - A Lisp that flows**

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

**並列、並行を簡単にできるのはQiのキモ** - スレッドセーフな設計と3層並行処理アーキテクチャ。

> **Note**: コーディングスタイルとフォーマット規則については [style-guide.md](../style-guide.md) を参照してください。

---

## 言語哲学 - Flow-Oriented Programming

### 核となる思想

**「データは流れ、プログラムは流れを設計する」**

Qiは**Flow-Oriented Programming**（流れ指向プログラミング）を体現します：

1. **データの流れが第一級市民**
   - パイプライン演算子 `|>` が言語の中心
   - `match` は流れを分岐・変換する制御構造
   - 小さな変換を組み合わせて大きな流れを作る
   - Unix哲学の「Do One Thing Well」を関数型で実現

2. **Simple, Fast, Concise**
   - **Simple**: 特殊形式9つ、記法最小限、学習曲線が緩やか
   - **Fast**: 軽量・高速起動・将来的にJITコンパイル
   - **Concise**: 短い関数名、パイプライン、`defn`糖衣構文で表現力豊か

3. **エネルギーの流れ**
   - データは一方向に流れる（左から右、上から下）
   - 副作用はタップ（`tap>`）で観察
   - 並列処理は流れの分岐・合流として表現
   - **並行・並列を簡単に** - スレッドセーフな設計で自然な並列化

4. **実用主義**
   - Lisp的純粋性より実用性を優先
   - モダンな機能（f-string、パターンマッチング）を積極採用
   - バッテリー同梱（豊富な文字列操作、ユーティリティ）

---

## 設計原則

1. **読みやすさ > 書きやすさ**
   - パイプラインは上から下、左から右に読める
   - データの流れが一目で分かる

2. **合成可能性**
   - 小さな関数を組み合わせて大きな処理を作る
   - 各ステップは独立してテスト可能

3. **段階的開示**
   - 初心者: 基本的な `|>` から始められる
   - 中級者: `match`、`loop`を活用
   - 上級者: 並列処理を駆使

4. **実行時の効率**
   - パイプラインは最適化される
   - 遅延評価で不要な計算を回避
   - 並列処理で自然にスケール

---

## ファイル拡張子

```
.qi
```

---

## 使い方 - Qiの実行方法

Qiは3つの実行モード + 標準入力をサポート：

### 1. REPL（対話モード）

```bash
qi
# または
qi -l utils.qi  # ファイルをプリロードしてREPL起動
```

### 2. スクリプトファイル実行

```bash
qi script.qi
qi examples/hello.qi
```

### 3. ワンライナー実行

```bash
qi -e '(println "Hello!")'
qi -e '(range 1 10 |> (map (fn [x] (* x x))) |> sum)'
```

### 4. 標準入力から実行

**Unix哲学に準拠 - パイプラインとの統合**

```bash
# echoから
echo '(println "Hello from stdin!")' | qi -

# heredocで複数行スクリプト
qi - <<'EOF'
(def data [1 2 3 4 5])
(def result (data |> (map (fn [x] (* x x))) |> sum))
(println (str "Sum of squares: " result))
EOF

# 他のコマンドからQiスクリプトを生成
jq -r '.script' config.json | qi -
curl -s https://example.com/script.qi | qi -

# 一時ファイル不要（セキュリティ向上）
echo "$SECRET_SCRIPT" | qi -

# CI/CDでの動的スクリプト実行
cat automation.qi | qi -
```

#### なぜ標準入力実行が重要か？

1. **Unix哲学** - 他のツール（python/node/ruby）も `-` をサポート
2. **パイプライン統合** - コマンドの出力を直接実行
3. **セキュリティ** - 機密スクリプトをファイルに残さない
4. **動的生成** - 他のツールがQiコードを生成→即実行
5. **CI/CD対応** - GitHub Secretsからスクリプト注入可能

```bash
# 実用例：JSONからQiスクリプトを抽出して実行
cat automation.json | jq -r '.tasks.cleanup.script' | qi -

# 実用例：動的にデータ処理スクリプトを生成
./generate-processor.sh --type=csv --columns=3 | qi -
```

---

## 基本設計

### 名前空間

**Lisp-1（Scheme派）** - 変数と関数は同じ名前空間

```qi
(def add (fn [x y] (+ x y)))
(def op add)           ;; 関数を変数に代入
(op 1 2)               ;; 3
```

### nil と bool

**nil と bool は別物、ただし条件式では nil も falsy**

```qi
nil false true          ;; 3つの異なる値
(if nil "yes" "no")     ;; "no" (nilはfalsy)
(if false "yes" "no")   ;; "no" (falseはfalsy)
(if 0 "yes" "no")       ;; "yes" (0はtruthy)
(if "" "yes" "no")      ;; "yes" (空文字もtruthy)

;; 明示的な比較
(= x nil)               ;; nilチェック
(= x false)             ;; falseチェック
```

### 型システム

Qiは動的型付け言語で、以下の基本型を持ちます：

- **数値**: 整数、浮動小数点数
- **文字列**: UTF-8対応、f-string対応
- **真偽値**: `true`, `false`
- **nil**: 値の不在を表す
- **ベクター**: `[1 2 3]`
- **マップ**: `{:key "value"}`
- **リスト**: `'(1 2 3)` (クオート必須)
- **関数**: 第一級オブジェクト
- **キーワード**: `:keyword`
