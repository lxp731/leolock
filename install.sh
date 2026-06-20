#!/bin/bash
set -e

# LeoLock 一键安装脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/lxp731/leolock/main/install.sh | bash

REPO="lxp731/leolock"
INSTALL_DIR="/usr/local/bin"

# ── 颜色 ────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'

info()  { echo -e "${CYAN}→${NC} $1"; }
ok()    { echo -e "${GREEN}✓${NC} $1"; }
err()   { echo -e "${RED}✗ $1${NC}"; exit 1; }

# ── 检测系统 ────────────────────────────────────────────────────
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)  PLATFORM="unknown-linux-gnu" ;;
        Darwin) PLATFORM="apple-darwin" ;;
        *)      err "不支持的操作系统: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             err "不支持的架构: $ARCH" ;;
    esac

    TARGET="${ARCH}-${PLATFORM}"
    info "检测到系统: $TARGET"
}

# ── 版本选择 ────────────────────────────────────────────────────
get_version() {
    if [ -n "$LEOLOCK_VERSION" ]; then
        VERSION="$LEOLOCK_VERSION"
    else
        info "获取最新版本..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
            | grep -o '"tag_name": *"v[^"]*"' \
            | grep -o 'v[0-9.]*' \
            | head -1)
        if [ -z "$VERSION" ]; then
            err "无法获取最新版本号，请手动指定: LEOLOCK_VERSION=1.2.0 bash install.sh"
        fi
        VERSION="${VERSION#v}"
    fi
    info "安装版本: v$VERSION"
}

# ── 下载 ────────────────────────────────────────────────────────
download() {
    local base="https://github.com/$REPO/releases/download/v$VERSION"
    local archive="leolock-$VERSION-$TARGET.tar.gz"
    local url="$base/$archive"

    TMPDIR=$(mktemp -d)
    info "下载: $url"
    curl -fsSL "$url" -o "$TMPDIR/$archive" || err "下载失败，请检查版本号是否正确"

    info "解压..."
    tar -xzf "$TMPDIR/$archive" -C "$TMPDIR"
}

# ── 安装 ────────────────────────────────────────────────────────
install_binaries() {
    if [ ! -f "$TMPDIR/leolock" ]; then
        err "未找到 leolock 二进制文件"
    fi

    if [ "$EUID" -ne 0 ] && [ ! -w "$INSTALL_DIR" ]; then
        info "需要 sudo 权限写入 $INSTALL_DIR"
        sudo cp "$TMPDIR/leolock" "$INSTALL_DIR/"
    else
        cp "$TMPDIR/leolock" "$INSTALL_DIR/"
    fi

    chmod +x "$INSTALL_DIR/leolock"
    ok "leolock → $INSTALL_DIR/leolock"
}

# ── 清理 ────────────────────────────────────────────────────────
cleanup() {
    rm -rf "$TMPDIR"
}

# ── 验证 ────────────────────────────────────────────────────────
verify() {
    echo ""
    if command -v leolock &>/dev/null; then
        leolock --version 2>/dev/null || ok "leolock 安装成功"
    fi
    echo ""
    echo "  快速开始:"
    echo "    leolock init          # 初始化"
    echo "    leolock encrypt <文件> # 加密"
}

# ── 主流程 ──────────────────────────────────────────────────────
main() {
    echo ""
    echo "  LeoLock 一键安装"
    echo "  ==============="
    echo ""

    detect_platform
    get_version
    download
    install_binaries
    cleanup
    verify
}

main
