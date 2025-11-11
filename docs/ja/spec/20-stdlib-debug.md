# デバッグ機能

Qiは組み込みのデバッグ機能を提供しており、トレース、ブレークポイント、スタックトレース取得などができます。

## 概要

デバッグモジュールは以下の機能を提供します：

- **トレース機能** - 関数の呼び出しと終了をログに出力
- **ブレークポイント** - 実行の一時停止
- **スタックトレース** - 現在のコールスタックの取得
- **デバッガ情報** - デバッガの状態確認

## デバッグ関数

### debug/trace

トレース機能を有効/無効にします。

```qi
(debug/trace enabled)
```

**引数:**
- `enabled` (boolean) - true=有効, false=無効

**戻り値:** nil

**例:**

```qi
(defn fibonacci [n]
  (if (<= n 1)
      n
      (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(debug/trace true)
(fibonacci 5)
(debug/trace false)
```

出力例:
```
[TRACE] -> fibonacci (test.qi:1)
[TRACE]   -> fibonacci (test.qi:1)
[TRACE]     -> fibonacci (test.qi:1)
[TRACE]     <- fibonacci
[TRACE]     -> fibonacci (test.qi:1)
[TRACE]     <- fibonacci
[TRACE]   <- fibonacci
[TRACE]   -> fibonacci (test.qi:1)
[TRACE]   <- fibonacci
[TRACE] <- fibonacci
```

### debug/break

ブレークポイントを設定します。デバッガがアタッチされている場合、実行が一時停止します。

```qi
(debug/break)
```

**引数:** なし

**戻り値:** nil

**例:**

```qi
(defn process-data [data]
  (println "Processing:" data)
  (debug/break)  ;; ここで一時停止
  (if (> data 100)
      "Large"
      "Small"))

(process-data 150)
```

### debug/stack

現在のコールスタックを文字列として取得します。

```qi
(debug/stack)
```

**引数:** なし

**戻り値:** string - スタックトレース

**例:**

```qi
(defn inner []
  (debug/stack))

(defn middle []
  (inner))

(defn outer []
  (middle))

(println (outer))
```

出力例:
```
  #0 inner at test.qi:2
  #1 middle at test.qi:5
  #2 outer at test.qi:8
```

### debug/info

デバッガの現在の状態を取得します。

```qi
(debug/info)
```

**引数:** なし

**戻り値:** map - デバッグ情報

返されるマップのキー：
- `:enabled` (boolean) - デバッガが有効かどうか
- `:state` (string) - デバッガの状態（"Running", "Paused", etc.）
- `:stack-depth` (integer) - 現在のスタックの深さ

**例:**

```qi
(println (debug/info))
;=> {:enabled false}

;; デバッガが有効な場合
;=> {:enabled true :state "Running" :stack-depth 0}
```

## 使用例

### エラー発生時のスタックトレース

```qi
(defn error-handler []
  (println "Error occurred at:")
  (println (debug/stack)))

(defn divide [a b]
  (if (= b 0)
      (error-handler)
      (/ a b)))

(divide 10 0)
```

### トレース機能を使ったデバッグ

```qi
(defn factorial [n]
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(debug/trace true)
(println "Result:" (factorial 5))
(debug/trace false)
```

### 条件付きブレークポイント

```qi
(defn process-batch [items]
  (each (fn [item]
          (when (> item 1000)
            (debug/break))  ;; 大きな値の時だけ停止
          (println "Processing:" item))
        items))

(process-batch [10 50 1200 5])
```

## VSCodeデバッグサポート

QiはVSCodeの統合デバッガをサポートしています：

- ブレークポイントのGUIでの設定
- ステップ実行（Step In, Step Over, Step Out）
- 変数のインスペクション
- ウォッチ式の評価
- コールスタックのナビゲーション

## デバッガの有効化

起動オプションでデバッガを有効化できます：

```bash
# デバッグモードで起動
qi --debug script.qi

# DAP経由でアタッチ可能（指定ポートでリッスン）
qi --debug-port 5678 script.qi
```

## パフォーマンス

デバッグ機能はパフォーマンスに影響を与えるため、プロダクション環境では使用しないでください：

- `debug/trace` を有効にすると、すべての関数呼び出しでログが出力されるため、実行速度が大幅に低下します
- `debug/break` や `debug/stack` は比較的軽量ですが、頻繁に呼び出すべきではありません
- `debug/info` は軽量で、パフォーマンスへの影響はほとんどありません

## 関連項目

- [テスト機能](14-stdlib-test.md) - テストとアサーション
- [プロファイリング](../../../ROADMAP.md#プロファイリング) - パフォーマンス測定
- [ログ機能](09-modules.md#log) - 構造化ログ
