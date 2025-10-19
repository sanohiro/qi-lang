#!/bin/bash
# @qi-docタグから言語要素一覧を抽出するスクリプト

echo "=== Qi Language Reference ==="
echo ""

# 1. 特殊形式
echo "## Special Forms"
echo ""
if rg -q '@qi-doc:special-forms' src/parser.rs; then
    rg '@qi-doc:(definition|function|binding|control-flow|pattern-matching|error-handling|macro|module)' src/parser.rs --no-filename | \
      sed 's/.*@qi-doc:/  - /' | sort -u
fi
echo ""

# 2. 演算子
echo "## Operators"
echo ""
if rg -q '@qi-doc:tokens' src/lexer.rs; then
    rg '@qi-doc:(pipe-operators|arrow-operators|pattern-operators|quote-operators|special-operators)' src/lexer.rs --no-filename | \
      sed 's/.*@qi-doc:/  - /' | sort -u
fi
echo ""

# 3. 頻出シンボル
echo "## Common Symbols"
echo ""
if rg -q '@qi-doc:common-symbols' src/intern.rs; then
    rg '@qi-doc:(io|collections|operators|accessors|predicates)' src/intern.rs --no-filename | \
      grep -v 'common-symbols\|common-keywords' | \
      sed 's/.*@qi-doc:/  - /' | sort -u
fi
echo ""

# 4. 頻出キーワード
echo "## Common Keywords"
echo ""
if rg -q '@qi-doc:common-keywords' src/intern.rs; then
    rg -A5 '@qi-doc:common-keywords' src/intern.rs --no-filename | \
      grep '@qi-doc:' | grep -v 'common-keywords' | \
      sed 's/.*@qi-doc:/  - /' | sort -u
fi
echo ""

# 5. 組み込み関数
echo "## Built-in Functions by Category"
echo ""
rg '@qi-doc:category' src/builtins/*.rs --no-filename | \
  sed 's/.*@qi-doc:category //' | \
  sort -u | \
  while IFS= read -r category; do
    echo "### $category"
    echo ""

    # このカテゴリの関数リストを取得
    rg -A1 "@qi-doc:category $category" src/builtins/*.rs | \
      grep '@qi-doc:functions' | \
      sed 's/.*@qi-doc:functions /  - /'

    echo ""
  done

echo "---"
echo ""
echo "Statistics:"
echo "  - Function categories: $(rg '@qi-doc:category' src/builtins/*.rs --no-filename | wc -l | tr -d ' ')"
echo "  - Tagged files: $(rg -l '@qi-doc:category' src/builtins/*.rs | wc -l | tr -d ' ')"
echo "  - Special forms: $(rg '@qi-doc:special-forms' src/parser.rs --no-filename | wc -l | tr -d ' ')"
echo "  - Operators: $(rg '@qi-doc:.*-operators' src/lexer.rs --no-filename | wc -l | tr -d ' ')"
