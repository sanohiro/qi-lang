#!/bin/bash

# リンクチェックスクリプト
# マークダウンファイル内のリンク（内部リンクと外部リンク）を検証します

PROJECT_ROOT="/Users/hiro/Projects/qi-lang"
cd "$PROJECT_ROOT"

echo "=== マークダウンリンクチェック ==="
echo ""

# 一時ファイル
INTERNAL_LINKS=$(mktemp)
EXTERNAL_LINKS=$(mktemp)
BROKEN_LINKS=$(mktemp)
ALL_LINKS=$(mktemp)

trap "rm -f $INTERNAL_LINKS $EXTERNAL_LINKS $BROKEN_LINKS $ALL_LINKS" EXIT

# docs配下とREADME、examplesなどのマークダウンファイルを対象
# targetやreleaseディレクトリは除外
echo "マークダウンファイルからリンクを抽出中..."

# rgを使ってリンクを抽出（[text](url)形式）
rg --no-heading --line-number '\[([^\]]+)\]\(([^)]+)\)' \
  --type md \
  --glob '!target/**' \
  --glob '!release/**' \
  --glob '!private/**' \
  -o -r '$2' \
  . > "$ALL_LINKS"

if [ ! -s "$ALL_LINKS" ]; then
  echo "❌ リンクが見つかりませんでした"
  exit 0
fi

# リンクを分類
cat "$ALL_LINKS" | while read -r line; do
  # ファイル名:行番号:URL の形式
  file=$(echo "$line" | cut -d: -f1)
  link=$(echo "$line" | cut -d: -f3-)

  # アンカーのみ（#で始まる）はスキップ
  if [[ "$link" =~ ^# ]]; then
    continue
  # 外部リンク（http/https）
  elif [[ "$link" =~ ^https?:// ]]; then
    echo "$file|$link" >> "$EXTERNAL_LINKS"
  # 内部リンク（相対パス）
  else
    echo "$file|$link" >> "$INTERNAL_LINKS"
  fi
done

# 内部リンクのチェック
echo "=== 内部リンクチェック ==="
if [ -s "$INTERNAL_LINKS" ]; then
  internal_count=$(wc -l < "$INTERNAL_LINKS" | xargs)
  echo "チェック対象: $internal_count 個の内部リンク"
  echo ""

  broken_count=0

  cat "$INTERNAL_LINKS" | while IFS='|' read -r file link; do
    # アンカー部分を除去（#以降）
    file_link=$(echo "$link" | sed 's/#.*//')

    if [ -z "$file_link" ]; then
      continue
    fi

    # 相対パスの解決
    # fileは./から始まるので、./を除去
    file_clean=$(echo "$file" | sed 's|^\./||')
    dir=$(dirname "$file_clean")

    # target_pathを構築（./を付けずに）
    if [ "$dir" = "." ]; then
      target_path="$file_link"
    else
      target_path="$dir/$file_link"
    fi

    # ファイルが存在するかチェック
    if [ ! -e "$target_path" ]; then
      echo "❌ BROKEN: $file_clean"
      echo "   リンク: $link"
      echo "   期待: $target_path"
      echo ""
      echo "x" >> "$BROKEN_LINKS"
      broken_count=$((broken_count + 1))
    fi
  done

  if [ -s "$BROKEN_LINKS" ]; then
    broken_total=$(wc -l < "$BROKEN_LINKS" | xargs)
    echo ""
    echo "❌ 壊れた内部リンク: $broken_total 個"
  else
    echo "✅ すべての内部リンクは有効です"
  fi
  echo ""
else
  echo "内部リンクが見つかりませんでした"
  echo ""
fi

# 外部リンクのチェック（curlで確認）
echo "=== 外部リンクチェック ==="
if [ -s "$EXTERNAL_LINKS" ]; then
  # 重複を除去
  unique_external=$(cat "$EXTERNAL_LINKS" | cut -d'|' -f2 | sort -u)
  total=$(echo "$unique_external" | wc -l | xargs)
  echo "チェック対象: $total 個のユニークな外部リンク"
  echo ""

  count=0
  failed_count=0

  echo "$unique_external" | while read -r url; do
    count=$((count + 1))
    printf "[%2d/%2d] %s\n" "$count" "$total" "$url"

    # HEAD リクエストでチェック（タイムアウト10秒）
    status_code=$(curl -s -o /dev/null -w "%{http_code}" -L --max-time 10 "$url" 2>/dev/null || echo "000")

    if [[ "$status_code" =~ ^[23] ]]; then
      echo "        ✅ OK ($status_code)"
    else
      echo "        ❌ NG ($status_code)"
      failed_count=$((failed_count + 1))
      echo "        使用箇所:"
      grep -F "$url" "$EXTERNAL_LINKS" | cut -d'|' -f1 | sed 's|^\./||' | sed 's/^/          - /'
      echo ""
    fi
  done
  echo ""
else
  echo "外部リンクが見つかりませんでした"
  echo ""
fi

echo "=== チェック完了 ==="
