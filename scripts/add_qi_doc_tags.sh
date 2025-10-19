#!/bin/bash
# @qi-docタグを一括追加するスクリプト

# 各ファイルのカテゴリ定義
declare -A categories
categories["string.rs"]="string"
categories["json.rs"]="data/json"
categories["yaml.rs"]="data/yaml"
categories["csv.rs"]="data/csv"
categories["http.rs"]="net/http"
categories["math.rs"]="math"
categories["core_collections.rs"]="core/collections"
categories["core_predicates.rs"]="core/predicates"
categories["hof.rs"]="core/functions"
categories["io.rs"]="io"
categories["path.rs"]="io/path"
categories["time.rs"]="util/time"
categories["util.rs"]="util"
categories["test.rs"]="test"
categories["log.rs"]="util/log"
categories["env.rs"]="util/env"

for file in "${!categories[@]}"; do
    filepath="src/builtins/$file"
    if [ -f "$filepath" ]; then
        category="${categories[$file]}"
        echo "Processing $file (category: $category)..."

        # FUNCTIONS配列の直前に@qi-docタグを挿入
        # 既にタグがある場合はスキップ
        if ! grep -q "@qi-doc:category" "$filepath"; then
            # "/// 登録すべき関数のリスト" の次の行にタグを挿入
            sed -i '' "/登録すべき関数のリスト/a\\
/// @qi-doc:category $category
" "$filepath"
        fi
    fi
done

echo "Done!"
