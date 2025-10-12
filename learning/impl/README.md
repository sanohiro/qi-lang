# 言語処理系実装ガイド

このディレクトリでは、Qi言語の実装を通じて**プログラミング言語処理系の設計と実装**を学びます。

言語処理系の実装は、コンピュータサイエンスの基礎的な分野であり、ここで学ぶ概念は他の言語処理系やコンパイラの理解にも役立ちます。

## 学習パス

1. **[概要](./01-overview.md)** - 言語処理系の全体像
2. **[字句解析](./02-lexer.md)** - ソースコードをトークンに分割
3. **[構文解析](./03-parser.md)** - トークンから構文木（AST）を構築
4. **[評価器](./04-evaluator.md)** - ASTを実行して結果を得る
5. **[環境とスコープ](./05-environment.md)** - 変数と関数の管理
6. **[ビルトイン関数](./06-builtins.md)** - ネイティブ関数の登録と実装

## Qi言語処理系のアーキテクチャ

```mermaid
flowchart TD
    Source[ソースコード]
    Lexer["字句解析器<br/>(lexer.rs)"]
    Tokens[トークン列]
    Parser["構文解析器<br/>(parser.rs)"]
    AST[抽象構文木<br/>(AST)]
    Eval["評価器<br/>(eval.rs)"]
    Result[実行結果]

    Source --> Lexer
    Lexer --> Tokens
    Tokens --> Parser
    Parser --> AST
    AST --> Eval
    Eval --> Result

    style Source fill:#e1f5ff
    style Lexer fill:#d4e9ff
    style Tokens fill:#fff4e1
    style Parser fill:#d4e9ff
    style AST fill:#ffe1f5
    style Eval fill:#d4e9ff
    style Result fill:#e1ffe1
```

## 対象読者

- プログラミング言語処理系の実装に興味がある方
- コンパイラやインタプリタの仕組みを学びたい方
- Lispの実装を通じて言語設計を理解したい方
- コンピュータサイエンスの基礎を実践的に学びたい方

## 前提知識

- 基本的なプログラミング知識
- 再帰的なデータ構造の理解（リスト、木構造）
- Rustの基本文法（learning/rust/を参照）

## 実装の特徴

Qiは**Lisp系言語**として以下の特徴を持ちます：

- **ホモイコニシティ**: コードとデータが同じ構造（S式）
- **動的型付け**: 実行時に型チェック
- **関数型プログラミング**: イミュータブルなデータ構造
- **マクロシステム**: コンパイル時のコード変換

これらの特徴により、言語処理系の本質的な概念を学びやすい設計になっています。
