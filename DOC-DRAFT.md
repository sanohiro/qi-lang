# Qi言語 — docstring多言語対応・パラメータ説明・外部参照仕様書

## 1. 概要
- Qi言語の `doc` マクロを拡張し、次の3形式のドキュメント付与をサポート
  1. **単言語文字列doc**（もっとも簡潔な形式）
  2. **構造化doc**（説明＋引数説明、単言語または多言語）
  3. **外部参照doc**（ネイティブdocはソース内、他言語はファイル参照）
- REPL/CLI `:doc 関数名` で現在の言語設定に応じた説明を表示
- 多言語対応は必要な箇所のみ。未対応なら単言語docとして扱う

---

## 2. 書式

### 2.1 単言語文字列doc
```
(def greet "Function to greet the user"
  (fn [name] (str "Hello, " name)))
```
- もっとも簡単な形式。docは文字列のみ
- `qi doc greet` → "Function to greet the user"

---

### 2.2 構造化doc（説明＋引数説明）

#### 単言語
```
(def greet
  (doc {:desc "Function to greet the user"
        :params {:name "User name"}})
  (fn [name] (str "Hello, " name)))
```

#### 多言語
```
(def greet
  (doc {:en {:desc "Function to greet the user"
             :params {:name "User name"}}
        :ja {:desc "ユーザーに挨拶する関数"
             :params {:name "ユーザー名"}}})
  (fn [name] (str "Hello, " name)))
```

---

### 2.3 外部参照doc（ネイティブ＋他言語ファイル）
```
(def greet
  (doc {:desc "Function to greet the user"
        :see-ja "docs/ja/greet-doc.qi"})
  (fn [name] (str "Hello, " name)))
```

外部ファイル `docs/ja/greet-doc.qi` の例:
```
(def greet-doc-ja {:desc "ユーザーに挨拶する関数"
                    :params {:name "ユーザー名"}})
```

---

## 3. `doc` マクロ仕様
```
(defmacro doc (info)
  ;; info: 文字列 or {:desc "..."} or {:lang {...}} or {:see-lang "..."}
  `(do
     (set-meta! *current-symbol* :doc ,info)
     ,info))
```

---

## 4. REPL `:doc` 表示処理例
```
(defn show-doc [sym lang]
  (let [info (get-meta sym :doc)]
    (cond
      ;; 単なる文字列doc
      (string? info) (print info)

      ;; 構造化または多言語doc
      (map? info)
        (cond
          ;; 外部ファイル参照
          (contains? info (keyword (str "see-" lang)))
            (let [path (get info (keyword (str "see-" lang)))]
              (when (file-exists? path)
                (let [alt (load-doc-file path)]
                  (when (:desc alt) (print (:desc alt)))
                  (when (:params alt)
                    (print "Parameters:")
                    (doseq [[k v] (:params alt)]
                      (print "  " k " - " v))))))

          ;; 多言語内蔵
          (contains? info lang)
            (let [entry (get info lang)]
              (when (:desc entry) (print (:desc entry)))
              (when (:params entry)
                (print "Parameters:")
                (doseq [[k v] (:params entry)]
                  (print "  " k " - " v))))

          ;; 単言語構造化
          (contains? info :desc)
            (do
              (print (:desc info))
              (when (:params info)
                (print "Parameters:")
                (doseq [[k v] (:params info)]
                  (print "  " k " - " v))))))))
```

---

## 5. ヘルパー `load-doc-file` の例
```
(defn load-doc-file [path]
  ;; ファイルをロードし、doc構造を返す
  (let [data (slurp path)]
    (eval (read-string data))))
```
※ 実際には安全な読み込みが望ましい

---

## 6. 動作仕様（フォールバック）
1. 指定言語に外部参照があれば読み込み
2. 多言語構造に指定言語があれば使用
3. 単言語構造化ならそれを使用
4. 単なる文字列ならそのまま表示
5. 該当なしなら第一言語を使用

---

## 7. 利点
- 単言語、多言語、外部参照の3方式を統一管理
- 小規模関数や短い説明はソース内に書きやすい
- 翻訳チームは外部docのみ編集可能
- REPL・CLIドキュメント閲覧が環境のLANG設定で自動切替

---

## 8. 推奨運用
- **基本**: ネイティブ言語のdocをソース内記述
- **詳細説明が必要な場合**: 構造化docで引数説明も記述
- **大規模多言語ドキュメント**: 外部参照`see-<lang>`で分離
- 