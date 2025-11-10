# 標準ライブラリ - 日時処理（time/）

**24以上の日時操作関数**

すべての関数は `time/` モジュールに属します。

---

## 現在時刻取得

### ISO 8601形式

```qi
;; time/now-iso - 現在時刻をISO 8601形式で取得
(time/now-iso)                           ;; => "2025-11-10T15:30:45.123456789+00:00"

;; time/today - 今日の日付をYYYY-MM-DD形式で取得
(time/today)                             ;; => "2025-11-10"
```

---

## Unix時刻変換

```qi
;; time/from-unix - Unixタイムスタンプ（秒）をISO 8601形式に変換
(time/from-unix 1699632000)              ;; => "2023-11-10T12:00:00+00:00"

;; time/to-unix - ISO 8601形式の文字列をUnixタイムスタンプ（秒）に変換
(time/to-unix "2023-11-10T12:00:00+00:00") ;; => 1699632000
```

---

## 日時フォーマット

### format - 任意のフォーマットで文字列化

```qi
;; time/format - タイムスタンプを指定フォーマットで文字列化
;; 第1引数: Unixタイムスタンプ（整数）またはISO 8601文字列
;; 第2引数: フォーマット文字列（strftime形式）

(time/format 1699632000 "%Y-%m-%d")      ;; => "2023-11-10"
(time/format 1699632000 "%Y年%m月%d日")  ;; => "2023年11月10日"
(time/format 1699632000 "%H:%M:%S")      ;; => "12:00:00"

;; ISO 8601文字列も受け付ける
(time/format "2023-11-10T12:00:00+00:00" "%Y/%m/%d %H:%M")
;; => "2023/11/10 12:00"

;; パイプラインで使用
(time/now-iso
 |> (time/format _ "%Y-%m-%d %H:%M:%S")) ;; => "2025-11-10 15:30:45"
```

### strftime形式の主要指定子

| 指定子 | 意味 | 例 |
|--------|------|-----|
| `%Y` | 年（4桁） | 2025 |
| `%m` | 月（2桁） | 11 |
| `%d` | 日（2桁） | 10 |
| `%H` | 時（24時間制、2桁） | 15 |
| `%I` | 時（12時間制、2桁） | 03 |
| `%M` | 分（2桁） | 30 |
| `%S` | 秒（2桁） | 45 |
| `%p` | AM/PM | PM |
| `%A` | 曜日（フル） | Sunday |
| `%a` | 曜日（略） | Sun |
| `%B` | 月名（フル） | November |
| `%b` | 月名（略） | Nov |

---

## 日時解析

### parse - フォーマット指定でパース

```qi
;; time/parse - フォーマット文字列を使って日付文字列をパース
;; 第1引数: 日付文字列
;; 第2引数: フォーマット文字列（strftime形式）
;; 戻り値: Unixタイムスタンプ（整数）

(time/parse "2023-11-10" "%Y-%m-%d")     ;; => 1699632000
(time/parse "2023年11月10日" "%Y年%m月%d日")
;; => 1699632000

(time/parse "10/11/2023 15:30" "%d/%m/%Y %H:%M")
;; => 1699632600
```

---

## 日時計算

### 加算

```qi
;; time/add-days - 日付に日数を加算
(time/add-days 1699632000 7)             ;; => 1700236800（7日後）

;; time/add-hours - 日付に時間を加算
(time/add-hours 1699632000 24)           ;; => 1699718400（24時間後）

;; time/add-minutes - 日付に分を加算
(time/add-minutes 1699632000 30)         ;; => 1699633800（30分後）

;; ISO 8601文字列も受け付ける
(time/add-days "2023-11-10T12:00:00+00:00" 1)
;; => 1699718400（翌日）

;; パイプラインで使用
(time/now-iso
 |> time/to-unix
 |> (time/add-days _ 7)
 |> (time/format _ "%Y-%m-%d")) ;; => "2025-11-17"（7日後）
```

### 減算

```qi
;; time/sub-days - 日付から日数を減算
(time/sub-days 1699632000 7)             ;; => 1699027200（7日前）

;; time/sub-hours - 日付から時間を減算
(time/sub-hours 1699632000 24)           ;; => 1699545600（24時間前）

;; time/sub-minutes - 日付から分を減算
(time/sub-minutes 1699632000 30)         ;; => 1699630200（30分前）
```

### 差分計算

```qi
;; time/diff-days - 2つの日付の差を日数で取得
(time/diff-days "2023-11-17T12:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 7（日）

;; time/diff-hours - 2つの日付の差を時間で取得
(time/diff-hours "2023-11-10T15:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 3（時間）

;; time/diff-minutes - 2つの日付の差を分で取得
(time/diff-minutes "2023-11-10T12:30:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 30（分）

;; 負の値も返す
(time/diff-days "2023-11-10T12:00:00+00:00" "2023-11-17T12:00:00+00:00")
;; => -7（date1がdate2より前）
```

---

## 日時比較

```qi
;; time/before? - date1がdate2より前か判定
(time/before? "2023-11-10T12:00:00+00:00" "2023-11-17T12:00:00+00:00")
;; => true

;; time/after? - date1がdate2より後か判定
(time/after? "2023-11-17T12:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => true

;; time/between? - dateがstart〜end内か判定（境界含む）
(time/between? "2023-11-15T12:00:00+00:00"
               "2023-11-10T12:00:00+00:00"
               "2023-11-20T12:00:00+00:00")
;; => true

;; Unixタイムスタンプも使用可能
(time/before? 1699632000 1700236800)     ;; => true
```

---

## 日時要素の抽出

```qi
;; time/year - 日付から年を取得
(time/year "2023-11-10T12:00:00+00:00")  ;; => 2023

;; time/month - 日付から月を取得（1-12）
(time/month "2023-11-10T12:00:00+00:00") ;; => 11

;; time/day - 日付から日を取得（1-31）
(time/day "2023-11-10T12:00:00+00:00")   ;; => 10

;; time/hour - 日付から時を取得（0-23）
(time/hour "2023-11-10T12:00:00+00:00")  ;; => 12

;; time/minute - 日付から分を取得（0-59）
(time/minute "2023-11-10T12:35:00+00:00") ;; => 35

;; time/second - 日付から秒を取得（0-59）
(time/second "2023-11-10T12:00:45+00:00") ;; => 45

;; time/weekday - 日付から曜日を取得（0=日曜, 1=月曜, ..., 6=土曜）
(time/weekday "2023-11-10T12:00:00+00:00") ;; => 5（金曜日）

;; Unixタイムスタンプも使用可能
(time/year 1699632000)                   ;; => 2023
```

---

## 実用例

### ログタイムスタンプ

```qi
;; 現在時刻でログ出力
(defn log [level msg]
  (println (str/format "[{}] {} - {}"
                       (time/format (time/now-iso) "%Y-%m-%d %H:%M:%S")
                       level
                       msg)))

(log "INFO" "Application started")
;; => "[2025-11-10 15:30:45] INFO - Application started"
```

### 日時計算パイプライン

```qi
;; 7日後の午前9時を計算
(time/now-iso
 |> time/to-unix
 |> (time/add-days _ 7)
 |> (time/format _ "%Y-%m-%d 09:00:00")
 |> (time/parse _ "%Y-%m-%d %H:%M:%S"))
;; => 1700467200（7日後の午前9時のUnixタイムスタンプ）

;; 営業日計算（土日を除く5日後）
(defn add-business-days [date days]
  (let [target (time/add-days date days)]
    (let [wd (time/weekday target)]
      (cond
        (= wd 0) (time/add-days target 1)  ;; 日曜→月曜
        (= wd 6) (time/add-days target 2)  ;; 土曜→月曜
        :else target))))

(add-business-days (time/to-unix "2023-11-10T12:00:00+00:00") 5)
;; => 週末を考慮した5営業日後
```

### 期限管理

```qi
;; タスクの締め切りチェック
(defn is-overdue? [due-date]
  (time/before? due-date (time/now-iso)))

(is-overdue? "2025-11-01T23:59:59+00:00") ;; => true

;; 締め切りまでの残り日数
(defn days-until [due-date]
  (time/diff-days due-date (time/now-iso)))

(days-until "2025-12-25T00:00:00+00:00") ;; => 45（残り45日）

;; 期限内か判定
(defn is-within-deadline? [start end]
  (time/between? (time/now-iso) start end))

(is-within-deadline? "2025-11-01T00:00:00+00:00" "2025-12-31T23:59:59+00:00")
;; => true
```

### データ集計

```qi
;; ログファイルから今日のエラーを抽出
(defn today-errors [logs]
  (let [today (time/today)]
    (logs
     |> (filter (fn [log]
              (and
                (str/contains? log "ERROR")
                (str/starts-with? log today)))))))

(today-errors ["2025-11-10 10:30 ERROR: DB connection failed"
               "2025-11-09 14:20 ERROR: Timeout"
               "2025-11-10 15:45 ERROR: Invalid input"])
;; => ["2025-11-10 10:30 ERROR: DB connection failed"
;;     "2025-11-10 15:45 ERROR: Invalid input"]
```

### 時刻範囲のフィルタリング

```qi
;; 特定期間のイベントを抽出
(defn filter-by-period [events start end]
  (events
   |> (filter (fn [evt]
            (time/between? (get evt :timestamp) start end)))))

(def events [{:timestamp "2023-11-10T10:00:00+00:00" :type "login"}
             {:timestamp "2023-11-15T14:30:00+00:00" :type "purchase"}
             {:timestamp "2023-11-20T09:00:00+00:00" :type "logout"}])

(filter-by-period events
                  "2023-11-10T00:00:00+00:00"
                  "2023-11-16T00:00:00+00:00")
;; => [{:timestamp "2023-11-10T10:00:00+00:00" :type "login"}
;;     {:timestamp "2023-11-15T14:30:00+00:00" :type "purchase"}]
```

### 曜日判定

```qi
;; 週末判定
(defn is-weekend? [date]
  (let [wd (time/weekday date)]
    (or (= wd 0) (= wd 6))))

(is-weekend? "2023-11-11T12:00:00+00:00") ;; => true（土曜日）
(is-weekend? "2023-11-10T12:00:00+00:00") ;; => false（金曜日）

;; 曜日名を取得
(def weekdays ["日" "月" "火" "水" "木" "金" "土"])

(defn weekday-name [date]
  (get weekdays (time/weekday date)))

(weekday-name "2023-11-10T12:00:00+00:00") ;; => "金"
```

### 日時バッチ処理

```qi
;; 1時間ごとのデータポイントを生成
(defn hourly-points [start-date hours]
  (stream/range 0 hours)
  |> (map (fn [h] (time/add-hours start-date h)))
  |> (map (fn [ts] (time/format ts "%Y-%m-%d %H:00:00")))
  |> collect)

(hourly-points (time/to-unix "2023-11-10T00:00:00+00:00") 24)
;; => ["2023-11-10 00:00:00" "2023-11-10 01:00:00" ... "2023-11-10 23:00:00"]
```

---

## タイムゾーンについて

**重要**: すべての時刻処理はUTC（協定世界時）で行われます。

### ISO 8601形式の時刻

- `time/now-iso` はUTCタイムゾーン（`+00:00`）で返します
- `time/format`, `time/parse` はUTCとして扱います
- ローカルタイムゾーンが必要な場合は、オフセットを手動で計算してください

```qi
;; UTCでの現在時刻
(time/now-iso)  ;; => "2025-11-10T15:30:45+00:00"

;; 日本時間（UTC+9）に変換したい場合
(time/now-iso
 |> time/to-unix
 |> (time/add-hours _ 9)
 |> (time/format _ "%Y-%m-%d %H:%M:%S JST"))
;; => "2025-11-11 00:30:45 JST"
```

---

## エラーハンドリング

### 無効な日時文字列

```qi
;; 無効なISO 8601形式
(time/to-unix "invalid-date")
;; => エラー: "time/to-unix: 無効な日付形式: invalid-date"

;; パース失敗
(time/parse "2023-13-45" "%Y-%m-%d")
;; => エラー: "time/parse: 日付文字列 '2023-13-45' を形式 '%Y-%m-%d' でパースできません"
```

### 無効なタイムスタンプ

```qi
;; 範囲外のUnixタイムスタンプ
(time/from-unix 99999999999999)
;; => エラー: "time/from-unix: 無効なタイムスタンプです"
```

### 型エラー

```qi
;; 文字列以外の引数
(time/to-unix 123)
;; => エラー: "time/to-unix: 文字列のみを受け付けます"

;; 整数以外の日数
(time/add-days "2023-11-10T12:00:00+00:00" "7")
;; => エラー: "time/add-days (days): 整数のみを受け付けます"
```

---

## 関数一覧

### 現在時刻取得
- `time/now-iso` - 現在時刻をISO 8601形式で取得
- `time/today` - 今日の日付をYYYY-MM-DD形式で取得

### Unix時刻変換
- `time/from-unix` - Unixタイムスタンプ→ISO 8601形式
- `time/to-unix` - ISO 8601形式→Unixタイムスタンプ

### フォーマット・パース
- `time/format` - タイムスタンプを指定フォーマットで文字列化
- `time/parse` - フォーマット文字列を使って日付文字列をパース

### 日時計算（加算）
- `time/add-days` - 日数を加算
- `time/add-hours` - 時間を加算
- `time/add-minutes` - 分を加算

### 日時計算（減算）
- `time/sub-days` - 日数を減算
- `time/sub-hours` - 時間を減算
- `time/sub-minutes` - 分を減算

### 日時計算（差分）
- `time/diff-days` - 2つの日付の差を日数で取得
- `time/diff-hours` - 2つの日付の差を時間で取得
- `time/diff-minutes` - 2つの日付の差を分で取得

### 日時比較
- `time/before?` - date1がdate2より前か判定
- `time/after?` - date1がdate2より後か判定
- `time/between?` - dateがstart〜end内か判定

### 日時要素抽出
- `time/year` - 年を取得
- `time/month` - 月を取得（1-12）
- `time/day` - 日を取得（1-31）
- `time/hour` - 時を取得（0-23）
- `time/minute` - 分を取得（0-59）
- `time/second` - 秒を取得（0-59）
- `time/weekday` - 曜日を取得（0=日曜, 1=月曜, ..., 6=土曜）

---

## 設計ノート

### ISO 8601形式の採用

すべての文字列ベースの日時表現はISO 8601形式（RFC 3339）を使用します。これにより：

- **タイムゾーン情報の明示**: `+00:00`のように常にタイムゾーンが含まれる
- **国際標準**: プログラム間のデータ交換が容易
- **ソート可能**: 文字列比較でも正しく順序付けできる
- **明確性**: 日付の曖昧さがない（MM/DD vs DD/MM等）

### Unixタイムスタンプとの互換性

- 外部システムとの連携やデータベース格納にはUnixタイムスタンプ（整数）が便利
- `time/to-unix`, `time/from-unix`で相互変換が可能
- 計算結果は基本的にUnixタイムスタンプで返す（パイプライン処理しやすい）

### 引数の柔軟性

多くの関数はUnixタイムスタンプとISO 8601文字列の両方を受け付けます：

```qi
;; 両方とも有効
(time/add-days 1699632000 7)
(time/add-days "2023-11-10T12:00:00+00:00" 7)
```

これにより、パイプライン処理がスムーズになります。
