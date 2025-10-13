# Qi言語 関数・特殊形式一覧

生成日時: 2025-10-13 09:47:10

## 概要

- **特殊形式**: 12個
- **Evaluatorが必要な関数**: 51個
- **通常の組み込み関数**: 385個
- **総計**: 448個

---

## 特殊形式 (12個)

Qi言語の構文として組み込まれている特殊な形式：

| 形式 | 説明 |
|------|------|
| `def` | 変数定義 |
| `fn` | 無名関数定義 |
| `let` | ローカルバインディング |
| `do` | 順次実行 |
| `if` | 条件分岐 |
| `match` | パターンマッチング |
| `try` | 例外処理 |
| `defer` | 遅延実行 |
| `\|>` | パイプライン演算子 |
| `\|\|>` | 並列パイプライン演算子 |
| `\|>?` | Railway Pipeline演算子 |
| `~>` | 非同期パイプライン演算子 |

また、内部的に使用される特殊処理：

| 形式 | 説明 |
|------|------|
| `and` | 論理AND（短絡評価） |
| `or` | 論理OR（短絡評価） |
| `quote` | クォート |
| `defn` | 名前付き関数定義（マクロ） |
| `_railway-pipe` | Railway Pipeline内部実装 |

---

## Evaluatorが必要な関数

これらの関数は引数として関数を受け取り、Evaluatorで評価する必要があります。

### Core (20個)

```
apply
branch
comp
drop-while
eval
every?
filter
find
go
map
map-lines
pmap
reduce
some?
swap!
take-while
tap
time
update
update-in
```

### list/モジュール (13個)

```
list/chunk
list/count-by
list/drop-last
list/find-index
list/group-by
list/keep
list/max-by
list/min-by
list/partition
list/partition-by
list/sort-by
list/split-at
list/sum-by
```

### map/モジュール (2個)

```
map/update-keys
map/update-vals
```

### async/モジュール (8個)

```
async/catch
async/parallel-do
async/pfilter
async/preduce
async/scope-go
async/select!
async/then
async/with-scope
```

### pipeline/モジュール (3個)

```
pipeline/filter
pipeline/map
pipeline/pipeline
```

### stream/モジュール (3個)

```
stream/filter
stream/iterate
stream/map
```

### test/モジュール (2個)

```
test/assert-throws
test/run
```

---

## 通常の組み込み関数

引数を評価するだけで実行できる関数（Evaluator不要）。

### Core (90個)

```
!=
%
*
+
-
/
<
<=
=
>
>=
abs
assoc
atom
atom?
chan
close!
coll?
concat
conj
cons
constantly
count
dec
deref
dissoc
distinct
drop
empty?
error
even?
false?
first
flatten
float?
function?
get
get-in
identity
inc
inspect
integer?
join
keys
keyword?
last
len
list?
macro?
map?
max
merge
min
negative?
nil?
not
now
nth
number?
odd?
partial
positive?
print
println
range
recv!
reset!
rest
reverse
send!
sequential?
sleep
some?
sort
split
str
string?
sum
take
timestamp
to-float
to-int
to-string
true?
uvar
vals
variable
vector?
zero?
zip
```

### list/モジュール (8個)

```
list/chunk
list/dedupe
list/drop-last
list/frequencies
list/interleave
list/split-at
list/take-nth
list/zipmap
```

### map/モジュール (3個)

```
map/assoc-in
map/dissoc-in
map/select-keys
```

### fn/モジュール (3個)

```
fn/complement
fn/juxt
fn/tap>
```

### math/モジュール (10個)

```
math/ceil
math/clamp
math/floor
math/pow
math/rand
math/rand-int
math/random-range
math/round
math/shuffle
math/sqrt
```

### str/モジュール (70個)

```
str/alnum?
str/alpha?
str/ascii?
str/blank?
str/bytes-count
str/camel
str/capitalize
str/chars
str/chars-count
str/contains?
str/digit?
str/drop
str/ends-with?
str/expand-tabs
str/format
str/format-comma
str/format-decimal
str/format-percent
str/from-base64
str/hash
str/html-decode
str/html-encode
str/indent
str/index-of
str/integer?
str/kebab
str/last-index-of
str/lines
str/lower
str/lower?
str/numeric?
str/pad
str/pad-left
str/pad-right
str/parse-float
str/parse-int
str/pascal
str/re-find
str/re-matches
str/re-replace
str/repeat
str/replace
str/replace-first
str/reverse
str/slice
str/slugify
str/snake
str/space?
str/splice
str/split-camel
str/squish
str/starts-with?
str/sub-after
str/sub-before
str/take
str/title
str/to-base64
str/trim
str/trim-left
str/trim-right
str/trunc-words
str/truncate
str/upper
str/upper?
str/url-decode
str/url-encode
str/uuid
str/word-count
str/words
str/wrap
```

### io/モジュール (19個)

```
io/append-file
io/copy-file
io/create-dir
io/delete-dir
io/delete-file
io/file-exists?
io/file-info
io/is-dir?
io/is-file?
io/list-dir
io/move-file
io/read-file
io/read-lines
io/temp-dir
io/temp-dir-keep
io/temp-file
io/temp-file-keep
io/write-file
io/write-stream
```

### path/モジュール (9個)

```
path/absolute
path/basename
path/dirname
path/extension
path/is-absolute?
path/is-relative?
path/join
path/normalize
path/stem
```

### env/モジュール (4個)

```
env/all
env/get
env/load-dotenv
env/set
```

### log/モジュール (6個)

```
log/debug
log/error
log/info
log/set-format
log/set-level
log/warn
```

### async/モジュール (7個)

```
async/all
async/await
async/cancel!
async/cancelled?
async/make-scope
async/race
async/try-recv!
```

### pipeline/モジュール (2個)

```
pipeline/fan-in
pipeline/fan-out
```

### stream/モジュール (8個)

```
stream/cycle
stream/drop
stream/file
stream/range
stream/realize
stream/repeat
stream/stream
stream/take
```

### json/モジュール (3個)

```
json/parse
json/pretty
json/stringify
```

### yaml/モジュール (3個)

```
yaml/parse
yaml/pretty
yaml/stringify
```

### csv/モジュール (5個)

```
csv/parse
csv/read-file
csv/read-stream
csv/stringify
csv/write-file
```

### http/モジュール (11個)

```
http/delete
http/get
http/get-stream
http/head
http/options
http/patch
http/post
http/post-stream
http/put
http/request
http/request-stream
```

### server/モジュール (16個)

```
server/json
server/no-content
server/not-found
server/ok
server/router
server/serve
server/static-dir
server/static-file
server/with-basic-auth
server/with-bearer
server/with-cache-control
server/with-compression
server/with-cors
server/with-json-body
server/with-logging
server/with-no-cache
```

### db/モジュール (18個)

```
db/begin
db/call
db/close
db/columns
db/commit
db/connect
db/driver-info
db/escape-like
db/exec
db/foreign-keys
db/indexes
db/query
db/query-one
db/rollback
db/sanitize
db/sanitize-identifier
db/supports?
db/tables
```

### set/モジュール (7個)

```
set/difference
set/disjoint?
set/intersect
set/subset?
set/superset?
set/symmetric-difference
set/union
```

### stats/モジュール (6個)

```
stats/mean
stats/median
stats/mode
stats/percentile
stats/stddev
stats/variance
```

### time/モジュール (25個)

```
time/add-days
time/add-hours
time/add-minutes
time/after?
time/before?
time/between?
time/day
time/diff-days
time/diff-hours
time/diff-minutes
time/format
time/from-unix
time/hour
time/minute
time/month
time/now-iso
time/parse
time/second
time/sub-days
time/sub-hours
time/sub-minutes
time/to-unix
time/today
time/weekday
time/year
```

### zip/モジュール (6個)

```
zip/add
zip/create
zip/extract
zip/gunzip
zip/gzip
zip/list
```

### args/モジュール (4個)

```
args/all
args/count
args/get
args/parse
```

### test/モジュール (5個)

```
test/assert
test/assert-eq
test/assert-not
test/clear
test/run-all
```

### profile/モジュール (4個)

```
profile/clear
profile/report
profile/start
profile/stop
```

### queue/モジュール (6個)

```
queue/dequeue
queue/empty?
queue/enqueue
queue/new
queue/peek
queue/size
```

### stack/モジュール (6個)

```
stack/empty?
stack/new
stack/peek
stack/pop
stack/push
stack/size
```

### markdown/モジュール (11個)

```
markdown/code-block
markdown/extract-code-blocks
markdown/header
markdown/image
markdown/join
markdown/link
markdown/list
markdown/ordered-list
markdown/parse
markdown/stringify
markdown/table
```

### cmd/モジュール (10個)

```
cmd/exec
cmd/interactive
cmd/lines
cmd/pipe
cmd/read-line
cmd/sh
cmd/stream-bytes
cmd/stream-lines
cmd/wait
cmd/write
```

---

## 使用方法

### REPLでの確認

```bash
# REPL起動
qi

# 環境に登録されている全ての変数を確認（env参照）
# 注: 現在のQiには組み込み関数一覧コマンドは未実装
```

### Pythonスクリプトでの抽出

このドキュメントは以下のように生成されました：

```python
# eval.rsのmatch文から抽出
import re
with open('src/eval.rs', 'r') as f:
    content = f.read()
pattern = r'match name\.as_str\(\) \{(.*?)\n\s+_ => \{\}'
# ...

# mod.rsのregister_native!から抽出
with open('src/builtins/mod.rs', 'r') as f:
    content = f.read()
pattern = r'"([^"]+)"\s*=>\s*[\w:]+::[\w_]+'
# ...
```

---

## 注意事項

- 特殊形式は構文の一部であり、通常の関数として呼び出すことはできません
- Evaluatorが必要な関数は、引数として関数を受け取り内部で評価します
- 通常の組み込み関数は、全ての引数を評価してから実行されます
- feature-gated関数（条件付きコンパイル）も含まれています
