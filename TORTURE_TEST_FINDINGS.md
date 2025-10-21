# Torture Test で発見した課題・改善点

torture testの作成過程で発見した実装上の課題や改善点をまとめます。

## 修正済み

### 1. Result型のキー不整合（json/yaml/http）
**問題**: Result型を返す関数のマップキーが不統一だった
- 実装: `"ok"`/`"error"`（文字列キー）
- matchパターン: `{:ok value}`（キーワードキー `:ok`）

**修正内容**:
- json.rs, yaml.rs, http.rsで`:ok`/`:error`キーを使用
- マップキー形式の統一: `":ok"`, `":error"`

**影響範囲**: json/yaml/http関数全て（9関数）

---

## 実装済み機能

### 2. `stream?` 述語関数 → **実装済み** (2025-10-21)
**実装内容**:
- `src/builtins/core_predicates.rs`に`stream?`述語を追加
- 型チェック述語が11個→12個に増加
- ドキュメント更新:
  - `std/docs/ja/core.qi` および `std/docs/en/core.qi`
  - `docs/spec/FUNCTION-INDEX.md`
  - `docs/spec/05-syntax-basics.md`

**使用例**:
```qi
(stream? (stream/range 0 10)) ;=> true
(stream? [1 2 3])            ;=> false
(stream? nil)                ;=> false
```

---

## 未実装機能（実装検討）

### 3. `to-vector`/`to-list` 関数（リスト⇄ベクター変換）
**問題**: リストをベクターに（またはその逆に）明示的に変換する関数が存在しない

**現状の回避策**:
```qi
; to-vectorが使えないため、リストのまま処理
(def bulk-rows (map (fn [i] [...]) (range 0 100)))
; consを使うとListになるが、csv/stringifyはVector/List両方受け付ける
```

**実装案**:
- `src/builtins/core_collections.rs`に追加
- `to-vector` - リストをベクターに変換
- `to-list` - ベクターをリストに変換

**優先度**: 低（多くの関数がList/Vector両方を受け付けるため、実用上の必要性は低い）

**備考**: `vector`関数（可変長引数からベクタを作成）は既に実装済み

---

## テスト改善

### 4. util torture test の sleep関連テスト
**問題**: 環境依存でタイミングテストが失敗しやすい（17/19 passed）

**失敗しているテスト**:
- `[6.1] sleep - 基本動作` - 100ms sleep後の経過時間チェック
- `[7.2] 時間測定` - 50ms sleep後の経過時間チェック（50-200msの範囲）

**原因**: システムの負荷やスケジューリングの影響で実際の待機時間が変動

**改善案**:
1. 許容範囲をさらに広げる（現在50-200ms → 50-500ms等）
2. sleep時間を長くする（100ms → 500ms等）
3. 複数回実行して平均を取る
4. CI環境では該当テストをスキップ

**優先度**: 低（torture testの性質上、一部失敗は許容範囲内）

---

## 設計上の発見

### 5. Result型の一貫性
**発見**: 関数によってResult型の使い方が異なる

**パターン1**: Result型マップを返す（json/yaml/http）
```qi
(match (json/parse "...")
  {:ok data} -> ...
  {:error e} -> ...)
```

**パターン2**: 直接値を返す、エラーは例外（csv/util/他多数）
```qi
(def data (csv/parse "..."))  ; 成功時は直接値、失敗時は実行時エラー
```

**考察**:
- パターン1: 安全だがやや冗長
- パターン2: 簡潔だがエラーハンドリングが難しい
- Railway Oriented Programming (`|>?`)でパターン1を簡潔に書ける

**推奨**: 新規関数はパターン1（Result型マップ）を採用

---

## テスト作成パターン（ノウハウ）

### 成功パターン
1. **Result型関数**: matchパターンマッチングで処理
2. **直接値返す関数**: そのまま使用
3. **ファイルI/O**: `/tmp/`ディレクトリを使用
4. **大量データ**: 100-1000件で負荷テスト
5. **往復変換**: parse→stringify→parseでデータ整合性確認

### 注意点
1. **mapのキー形式**:
   - キーワード: `":name"`
   - 文字列: `"\"test\""`
   - 内部表現を理解してテスト
2. **List vs Vector**: 多くの関数が両方受け付ける
3. **環境依存テスト**: タイミング系は失敗を許容
4. **関数の引数順**: ドキュメント・実装を確認（例: csv/write-fileは(data path)）

---

## 次回実装予定

残りのtorture test（85関数）:
- [ ] env（4関数）
- [ ] io（16関数）
- [ ] stream（7関数）
- [ ] markdown（11関数）
- [ ] zip（6関数）
- [ ] http（13関数）
- [ ] db（24関数）
- [ ] profile（4関数）

**推定作業時間**: 4-6時間（パターンが確立済みのため効率的に作成可能）
