#!/bin/bash
# CMS APIテストスクリプト

set -e

BASE_URL="http://localhost:3000"
TOKEN=""

echo "=== Qi CMS API Test ==="
echo

# カラー出力
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 成功メッセージ
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# エラーメッセージ
error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# 1. ユーザー登録
echo "1. ユーザー登録..."
RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123","role":"admin"}')

if echo "$RESPONSE" | grep -q "User created"; then
    success "ユーザー登録成功"
else
    error "ユーザー登録失敗: $RESPONSE"
fi
echo

# 2. ログイン
echo "2. ログイン..."
RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123"}')

TOKEN=$(echo "$RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    error "ログイン失敗: $RESPONSE"
else
    success "ログイン成功 (Token取得)"
fi
echo

# 3. 記事作成
echo "3. 記事作成..."
RESPONSE=$(curl -s -X POST "$BASE_URL/api/posts" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Test Post","content":"This is a test post from Qi CMS."}')

if echo "$RESPONSE" | grep -q "Post created"; then
    success "記事作成成功"
else
    error "記事作成失敗: $RESPONSE"
fi
echo

# 4. 記事一覧取得
echo "4. 記事一覧取得..."
RESPONSE=$(curl -s "$BASE_URL/api/posts")

if echo "$RESPONSE" | grep -q "posts"; then
    success "記事一覧取得成功"
    echo "$RESPONSE" | head -c 200
    echo "..."
else
    error "記事一覧取得失敗: $RESPONSE"
fi
echo
echo

# 5. 記事詳細取得（スラッグ）
echo "5. 記事詳細取得..."
RESPONSE=$(curl -s "$BASE_URL/api/posts/test-post")

if echo "$RESPONSE" | grep -q "Test Post"; then
    success "記事詳細取得成功"
else
    error "記事詳細取得失敗: $RESPONSE"
fi
echo

# 6. コメント作成
echo "6. コメント作成..."
RESPONSE=$(curl -s -X POST "$BASE_URL/api/posts/1/comments" \
  -H "Content-Type: application/json" \
  -d '{"author_name":"Tester","content":"Great CMS!"}')

if echo "$RESPONSE" | grep -q "Comment created"; then
    success "コメント作成成功"
else
    error "コメント作成失敗: $RESPONSE"
fi
echo

# 7. 記事更新
echo "7. 記事更新..."
RESPONSE=$(curl -s -X PUT "$BASE_URL/api/posts/1" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Updated Test Post","content":"Updated content","status":"published"}')

if echo "$RESPONSE" | grep -q "Post updated"; then
    success "記事更新成功"
else
    error "記事更新失敗: $RESPONSE"
fi
echo

# 8. 記事削除
echo "8. 記事削除..."
RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/posts/1" \
  -H "Authorization: Bearer $TOKEN")

if echo "$RESPONSE" | grep -q "Post deleted"; then
    success "記事削除成功"
else
    error "記事削除失敗: $RESPONSE"
fi
echo

echo "=== All tests passed! ==="
