#!/bin/bash
set -e

# リリースアーカイブ作成スクリプト
# Usage: ./scripts/release.sh [--target TARGET] [--all]
#   --target: Rust target triple (e.g., x86_64-pc-windows-gnu, x86_64-unknown-linux-gnu)
#   --all: Build for all supported platforms

# カラー出力
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Qi Release Archive Builder ===${NC}"

# バージョン番号を取得
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${GREEN}Version: ${VERSION}${NC}"

# 引数解析
CROSS_TARGET=""
BUILD_ALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            CROSS_TARGET="$2"
            shift 2
            ;;
        --all)
            BUILD_ALL=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--target TARGET] [--all]"
            exit 1
            ;;
    esac
done

# 全プラットフォームビルド
if [ "$BUILD_ALL" = true ]; then
    echo -e "${YELLOW}Building for all platforms...${NC}"

    # ホストプラットフォーム
    "$0"

    # Windows (x86_64)
    if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        "$0" --target x86_64-pc-windows-gnu
    else
        echo -e "${YELLOW}Skipping Windows build (mingw-w64 not installed)${NC}"
    fi

    # Linux (x86_64) - macOSの場合
    if [[ "$(uname -s)" == "Darwin" ]] && command -v cross &> /dev/null; then
        "$0" --target x86_64-unknown-linux-gnu
    fi

    exit 0
fi

# クロスコンパイルの場合
if [ -n "$CROSS_TARGET" ]; then
    echo -e "${BLUE}Cross-compiling for: ${CROSS_TARGET}${NC}"

    # ターゲット追加チェック
    if ! rustup target list --installed | grep -q "$CROSS_TARGET"; then
        echo -e "${YELLOW}Adding target ${CROSS_TARGET}...${NC}"
        rustup target add "$CROSS_TARGET"
    fi

    # ビルド
    echo -e "${BLUE}Building release binary for ${CROSS_TARGET}...${NC}"
    cargo build --release --target "$CROSS_TARGET"

    # プラットフォーム名とアーキテクチャを決定
    case "$CROSS_TARGET" in
        x86_64-pc-windows-gnu|x86_64-pc-windows-msvc)
            PLATFORM="windows"
            ARCH_NAME="x86_64"
            BINARY_EXT=".exe"
            ;;
        x86_64-unknown-linux-gnu)
            PLATFORM="linux"
            ARCH_NAME="x86_64"
            BINARY_EXT=""
            ;;
        aarch64-unknown-linux-gnu)
            PLATFORM="linux"
            ARCH_NAME="arm64"
            BINARY_EXT=""
            ;;
        x86_64-apple-darwin)
            PLATFORM="darwin"
            ARCH_NAME="x86_64"
            BINARY_EXT=""
            ;;
        aarch64-apple-darwin)
            PLATFORM="darwin"
            ARCH_NAME="arm64"
            BINARY_EXT=""
            ;;
        *)
            echo "Unsupported target: $CROSS_TARGET"
            exit 1
            ;;
    esac

    TARGET="${PLATFORM}-${ARCH_NAME}"
    BINARY_PATH="target/${CROSS_TARGET}/release/qi${BINARY_EXT}"
else
    # ホストプラットフォーム検出
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Darwin)
            PLATFORM="darwin"
            ;;
        Linux)
            PLATFORM="linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows"
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
    echo -e "${GREEN}Host Target: ${TARGET}${NC}"

    # ビルド
    echo -e "${BLUE}Building release binary...${NC}"
    cargo build --release

    if [ "$PLATFORM" = "windows" ]; then
        BINARY_EXT=".exe"
    else
        BINARY_EXT=""
    fi
    BINARY_PATH="target/release/qi${BINARY_EXT}"
fi

# リリースディレクトリ作成
RELEASE_DIR="release"
ARCHIVE_NAME="qi-v${VERSION}-${TARGET}"
TEMP_DIR="${RELEASE_DIR}/${ARCHIVE_NAME}"

mkdir -p "${TEMP_DIR}/qi"

# ファイルをコピー
echo -e "${BLUE}Copying files...${NC}"
cp "${BINARY_PATH}" "${TEMP_DIR}/qi/"
cp -r std "${TEMP_DIR}/qi/"

# README等も含める（オプション）
if [ -f README.md ]; then
    cp README.md "${TEMP_DIR}/qi/"
fi
if [ -f LICENSE ]; then
    cp LICENSE "${TEMP_DIR}/qi/"
fi

# アーカイブ作成（プラットフォームに応じて形式を変更）
echo -e "${BLUE}Creating archive...${NC}"
cd "${RELEASE_DIR}"

if [ "$PLATFORM" = "windows" ]; then
    # Windows用にZIPアーカイブを作成
    if command -v zip &> /dev/null; then
        zip -r "${ARCHIVE_NAME}.zip" "${ARCHIVE_NAME}/qi"
        ARCHIVE_FILE="${ARCHIVE_NAME}.zip"
    else
        echo -e "${YELLOW}Warning: zip not found, creating tar.gz instead${NC}"
        tar czf "${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}/qi"
        ARCHIVE_FILE="${ARCHIVE_NAME}.tar.gz"
    fi
else
    # Unix系はtar.gz
    tar czf "${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}/qi"
    ARCHIVE_FILE="${ARCHIVE_NAME}.tar.gz"
fi

cd ..

# チェックサム生成
echo -e "${BLUE}Generating checksum...${NC}"
if command -v shasum &> /dev/null; then
    shasum -a 256 "${RELEASE_DIR}/${ARCHIVE_FILE}" > "${RELEASE_DIR}/${ARCHIVE_FILE}.sha256"
elif command -v sha256sum &> /dev/null; then
    sha256sum "${RELEASE_DIR}/${ARCHIVE_FILE}" > "${RELEASE_DIR}/${ARCHIVE_FILE}.sha256"
fi

# サイズ確認
SIZE=$(ls -lh "${RELEASE_DIR}/${ARCHIVE_FILE}" | awk '{print $5}')

echo ""
echo -e "${GREEN}✓ Release archive created:${NC}"
echo -e "  ${RELEASE_DIR}/${ARCHIVE_FILE} (${SIZE})"
echo ""

# インストール手順を表示（プラットフォームに応じて変更）
if [ "$PLATFORM" = "windows" ]; then
    echo -e "${BLUE}Installation (Windows):${NC}"
    if [[ "$ARCHIVE_FILE" == *.zip ]]; then
        echo "  1. Extract ${ARCHIVE_FILE}"
        echo "  2. Move 'qi' folder to C:\\Program Files\\"
        echo "  3. Add C:\\Program Files\\qi to PATH"
    else
        echo "  1. Extract ${ARCHIVE_FILE} (use 7-Zip or similar)"
        echo "  2. Move 'qi' folder to C:\\Program Files\\"
        echo "  3. Add C:\\Program Files\\qi to PATH"
    fi
    echo ""
    echo -e "${BLUE}Test:${NC}"
    echo "  qi.exe --version"
else
    echo -e "${BLUE}Installation (Unix):${NC}"
    echo "  tar xzf ${ARCHIVE_FILE}"
    echo "  sudo mv ${ARCHIVE_NAME}/qi /usr/local/"
    echo "  export PATH=\"/usr/local/qi:\$PATH\""
    echo ""
    echo -e "${BLUE}Test:${NC}"
    echo "  /usr/local/qi/qi --version"
fi
echo ""
