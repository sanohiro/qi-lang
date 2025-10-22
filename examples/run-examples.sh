#!/bin/bash
# Qi Examples Runner with Docker
#
# このスクリプトは、PostgreSQL/MySQL/Redis環境をDockerで起動し、
# サンプルコードを実行します。
#
# 使用方法:
#   ./run-examples.sh postgres   # PostgreSQLサンプルを実行
#   ./run-examples.sh mysql      # MySQLサンプルを実行
#   ./run-examples.sh kvs        # KVS/Redisサンプルを実行
#   ./run-examples.sh all        # すべてのサンプルを実行
#   ./run-examples.sh cleanup    # Dockerコンテナを停止・削除

set -e

# プロジェクトルートに移動
cd "$(dirname "$0")/.."

# カラー出力
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}ℹ ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

# Qiバイナリをビルド
build_qi() {
    if [ ! -f "target/debug/qi" ]; then
        info "Qiをビルドしています..."
        cargo build
        success "ビルド完了"
    else
        info "Qiバイナリが見つかりました"
    fi
}

# Docker Composeコマンドを取得
get_docker_compose_cmd() {
    if command -v docker-compose &> /dev/null; then
        echo "docker-compose"
    elif docker compose version &> /dev/null; then
        echo "docker compose"
    else
        error "Docker Composeが見つかりません"
        exit 1
    fi
}

# Dockerコンテナを起動
start_containers() {
    local service=$1
    local compose_cmd=$(get_docker_compose_cmd)
    info "Dockerコンテナを起動中: $service"

    cd examples
    if [ "$service" = "all" ]; then
        $compose_cmd up -d
    else
        $compose_cmd up -d "$service"
    fi
    cd ..

    # ヘルスチェック待機
    info "サービスの起動を待機中..."
    sleep 5

    success "コンテナ起動完了"
}

# PostgreSQLサンプル実行
run_postgres_example() {
    info "PostgreSQLサンプルを実行中..."
    start_containers "postgres"

    echo ""
    ./target/debug/qi examples/18-postgresql.qi
    echo ""

    success "PostgreSQLサンプル完了"
}

# MySQLサンプル実行
run_mysql_example() {
    info "MySQLサンプルを実行中..."
    start_containers "mysql"

    # MySQLの起動待機（少し時間がかかる）
    info "MySQLの完全起動を待機中..."
    sleep 10

    echo ""
    ./target/debug/qi examples/20-mysql.qi
    echo ""

    success "MySQLサンプル完了"
}

# KVS/Redisサンプル実行
run_kvs_example() {
    info "KVS/Redisサンプルを実行中..."
    start_containers "redis"

    echo ""
    ./target/debug/qi examples/21-kvs-unified.qi
    echo ""

    success "KVS/Redisサンプル完了"
}

# すべてのサンプル実行
run_all_examples() {
    info "すべてのサンプルを実行します"
    start_containers "all"

    # MySQLの起動待機
    info "MySQLの完全起動を待機中..."
    sleep 10

    echo ""
    echo "========================================="
    echo "PostgreSQL サンプル"
    echo "========================================="
    ./target/debug/qi examples/18-postgresql.qi

    echo ""
    echo "========================================="
    echo "MySQL サンプル"
    echo "========================================="
    ./target/debug/qi examples/20-mysql.qi

    echo ""
    echo "========================================="
    echo "KVS/Redis サンプル"
    echo "========================================="
    ./target/debug/qi examples/21-kvs-unified.qi

    echo ""
    success "すべてのサンプル完了"
}

# クリーンアップ
cleanup() {
    local compose_cmd=$(get_docker_compose_cmd)
    info "Dockerコンテナを停止・削除中..."
    cd examples
    $compose_cmd down -v
    cd ..
    success "クリーンアップ完了"
}

# メイン処理
main() {
    local command=${1:-help}

    case "$command" in
        postgres)
            build_qi
            run_postgres_example
            ;;
        mysql)
            build_qi
            run_mysql_example
            ;;
        kvs|redis)
            build_qi
            run_kvs_example
            ;;
        all)
            build_qi
            run_all_examples
            ;;
        cleanup|clean|down)
            cleanup
            ;;
        help|--help|-h)
            echo "使用方法: $0 <command>"
            echo ""
            echo "コマンド:"
            echo "  postgres   PostgreSQLサンプルを実行"
            echo "  mysql      MySQLサンプルを実行"
            echo "  kvs        KVS/Redisサンプルを実行"
            echo "  all        すべてのサンプルを実行"
            echo "  cleanup    Dockerコンテナを停止・削除"
            echo "  help       このヘルプを表示"
            echo ""
            echo "例:"
            echo "  $0 postgres"
            echo "  $0 all"
            echo "  $0 cleanup"
            ;;
        *)
            error "不明なコマンド: $command"
            echo "使用方法: $0 help"
            exit 1
            ;;
    esac
}

main "$@"
