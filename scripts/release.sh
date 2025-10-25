#!/bin/bash
set -e

# リリースアーカイブ作成スクリプト
# Usage: ./scripts/release.sh

# カラー出力
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Qi Release Archive Builder ===${NC}"

# バージョン番号を取得
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${GREEN}Version: ${VERSION}${NC}"

# プラットフォーム検出
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Darwin)
        PLATFORM="darwin"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

TARGET="${PLATFORM}-${ARCH_NAME}"
echo -e "${GREEN}Target: ${TARGET}${NC}"

# ビルド
echo -e "${BLUE}Building release binary...${NC}"
cargo build --release

# リリースディレクトリ作成
RELEASE_DIR="release"
ARCHIVE_NAME="qi-v${VERSION}-${TARGET}"
TEMP_DIR="${RELEASE_DIR}/${ARCHIVE_NAME}"

rm -rf "${RELEASE_DIR}"
mkdir -p "${TEMP_DIR}/qi"

# ファイルをコピー
echo -e "${BLUE}Copying files...${NC}"
cp target/release/qi "${TEMP_DIR}/qi/"
cp -r std "${TEMP_DIR}/qi/"

# README等も含める（オプション）
if [ -f README.md ]; then
    cp README.md "${TEMP_DIR}/qi/"
fi
if [ -f LICENSE ]; then
    cp LICENSE "${TEMP_DIR}/qi/"
fi

# アーカイブ作成
echo -e "${BLUE}Creating archive...${NC}"
cd "${RELEASE_DIR}"
tar czf "${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}/qi"
cd ..

# チェックサム生成
echo -e "${BLUE}Generating checksum...${NC}"
if command -v shasum &> /dev/null; then
    shasum -a 256 "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz" > "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz.sha256"
elif command -v sha256sum &> /dev/null; then
    sha256sum "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz" > "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz.sha256"
fi

# サイズ確認
SIZE=$(ls -lh "${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz" | awk '{print $5}')

echo ""
echo -e "${GREEN}✓ Release archive created:${NC}"
echo -e "  ${RELEASE_DIR}/${ARCHIVE_NAME}.tar.gz (${SIZE})"
echo ""
echo -e "${BLUE}Installation:${NC}"
echo "  tar xzf ${ARCHIVE_NAME}.tar.gz"
echo "  sudo mv ${ARCHIVE_NAME}/qi /usr/local/"
echo "  export PATH=\"/usr/local/qi:\$PATH\""
echo ""
echo -e "${BLUE}Test:${NC}"
echo "  /usr/local/qi/qi --version"
