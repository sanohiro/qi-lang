=== Qi Language Reference ===

## Special Forms

  - binding let
  - control-flow if, do, when, while, until, while-some, until-error, loop, recur
  - definition def, defn, defn-
  - error-handling try, defer
  - function fn
  - macro mac
  - module module, export, use, flow
  - pattern-matching match

## Operators

  - arrow-operators ->, =>
  - pattern-operators |
  - pipe-operators |>, |>?, ||>, ~>
  - quote-operators ', `, ,, ,@
  - special-operators @, ...

## Common Symbols

  - accessors get, assoc
  - collections list, vector, map, filter, reduce, first, rest, cons, concat
  - io print, println
  - operators +, -, *, /, =, <, >, <=, >=, not=
  - predicates number?, fn?, string?, list?, vector?, map?, nil?, empty?

## Common Keywords

  - data name, value, id, type, title, description, data
  - http status, message, body, headers, method, path, query, params, request, response
  - result ok, error
  - time created, updated, timestamp

## Built-in Functions by Category

### args

  - all, get, parse, count

### auth/jwt

  - jwt/sign, jwt/verify, jwt/decode

### auth/password

  - password/hash, password/verify

### cmd

  - exec, sh, pipe, pipe!, lines, stream-lines, stream-bytes, interactive, write, read-line, wait

### core/collections

  - first, rest, last, nth, len, count, cons, conj, concat, reverse, sort, distinct, range, repeat, take, drop, zip, etc.

### core/functions

  - identity, constantly, partial, comp, apply

### core/io-logic

  - print, println, not, error

### core/numeric

  - +, -, *, /, %, abs, min, max, inc, dec, sum, =, <, >, <=, >=

### core/predicates

  - nil?, list?, vector?, map?, string?, integer?, float?, number?, keyword?, function?, atom?, coll?, sequential?, empty?, some?, true?, false?, error?, even?, odd?, positive?, negative?, zero?

### core/state-meta

  - atom, deref, swap!, reset!, eval, uvar, variable, macro?

### core/string

  - str, split, join

### core/util

  - to-int, to-float, to-string, now, timestamp, sleep

### data/csv

  - parse, stringify, read-file, write-file, read-stream

### data/json

  - parse, stringify, pretty

### data/yaml

  - parse, stringify, pretty

### db

  - connect, close, exec, query, query-one, prepare, exec-prepared, query-prepared, begin, commit, rollback, escape-string, escape-identifier, table-list, column-list, table-exists?, column-exists?, create-table, drop-table, add-column, drop-column, list-indexes, create-index, drop-index

### debug

  - trace, break, stack, info

### ds

  - queue/new, queue/enqueue, queue/dequeue, queue/peek, queue/empty?, queue/size, stack/new, stack/push, stack/pop, stack/peek, stack/empty?, stack/size

### env

  - get, set, all, load-dotenv

### flow/control

  - branch

### fn

  - map, filter, reduce, pmap, pfilter, preduce, partition, group-by, map-lines, update, update-in, count-by, complement, juxt, tap>, tap

### go

  - chan, send!, recv!, close!, chan-closed?, then, catch, go, pipeline, pipeline-map, pipeline-filter, select!, atom, swap!, reset!, deref, scope, scope-go, with-scope, parallel-do

### io

  - temp-file, temp-dir, cleanup-temp
  - read-file, write-file, append-file, read-lines, file-exists?, file-stream, write-stream, list-dir, create-dir, delete-file, delete-dir, copy-file, move-file, file-info, is-file?, is-dir?, stdin-line, stdin-lines

### io/temp

  - temp-file, temp-dir, cleanup-temp

### kvs

  - kvs/redis-get, kvs/redis-set, kvs/redis-delete, kvs/redis-exists?, kvs/redis-keys, kvs/redis-expire, kvs/redis-ttl, kvs/redis-incr, kvs/redis-decr, kvs/redis-lpush, kvs/redis-rpush, kvs/redis-lpop, kvs/redis-rpop, kvs/redis-lrange, kvs/redis-hset, kvs/redis-hget, kvs/redis-hgetall, kvs/redis-sadd, kvs/redis-smembers, kvs/redis-mget, kvs/redis-mset
  - kvs/connect, kvs/get, kvs/set, kvs/del, kvs/exists, kvs/keys, kvs/expire, kvs/ttl, kvs/incr, kvs/decr, kvs/lpush, kvs/rpush, kvs/lpop, kvs/rpop, kvs/lrange, kvs/hset, kvs/hget, kvs/hgetall, kvs/sadd, kvs/smembers, kvs/mget, kvs/mset

### kvs/redis

  - kvs/redis-get, kvs/redis-set, kvs/redis-delete, kvs/redis-exists?, kvs/redis-keys, kvs/redis-expire, kvs/redis-ttl, kvs/redis-incr, kvs/redis-decr, kvs/redis-lpush, kvs/redis-rpush, kvs/redis-lpop, kvs/redis-rpop, kvs/redis-lrange, kvs/redis-hset, kvs/redis-hget, kvs/redis-hgetall, kvs/redis-sadd, kvs/redis-smembers, kvs/redis-mget, kvs/redis-mset

### list

  - take-while, drop-while, split-at, interleave, frequencies, sort-by, chunk, max-by, min-by, sum-by, find, find-index, every?, some?, zipmap, partition-by, take-nth, keep, dedupe, drop-last

### log

  - debug, info, warn, error, set-level, set-format

### map

  - select-keys, assoc-in, dissoc-in, update-keys, update-vals

### markdown

  - header, bold, italic, code, codeblock, link, list, table, quote, hr, escape

### math

  - pow, sqrt, round, floor, ceil, clamp

### net/http

  - get, post, put, delete, patch, head, options, request, get-stream, post-stream, request-stream

### path

  - join, basename, dirname, extension, stem, absolute, normalize, is-absolute?, is-relative?

### profile

  - enable, disable, reset, report

### server

  - serve, router, ok, json, not-found, no-content, with-logging, with-cors, with-json-body, static-file, static-dir

### set

  - union, intersection, difference, subset?, superset?

### stats

  - mean, median, mode, stddev, variance, min, max, sum, product, percentile

### stream

  - stream, range, iterate, repeat, cycle, map, filter, take, drop, realize

### string


### test

  - assert-eq, assert-ne, assert-true, assert-false, assert-nil, run, assert-throws, summary, clear

### time

  - now-iso, from-unix, to-unix, format, today, add-days, add-hours, add-minutes, sub-days, sub-hours, sub-minutes, diff-days, diff-hours, diff-minutes, before?, after?, between?, parse, year, month, day, hour, minute, second, weekday

### util

  - inspect

### validation

  - validate

### table

  - select, order-by, take, drop, where

### zip

  - create, extract, list, gzip, gunzip

---

Statistics:
  - Function categories: 49
  - Tagged files: 45
  - Special forms: 1
  - Operators: 5
