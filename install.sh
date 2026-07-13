#!/usr/bin/env bash
set -euo pipefail

REPO="pr4j3sh/hippo"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="hippo"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}==> $1${NC}"; }
warn()  { echo -e "${YELLOW}==> $1${NC}"; }
error() { echo -e "${RED}==> $1${NC}" >&2; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        *)       error "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)   arch="aarch64" ;;
        *)               error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

# Get latest release tag from GitHub
get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    local version
    version=$(curl -fsSL "$url" | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    if [ -z "$version" ]; then
        error "Failed to get latest release version"
        exit 1
    fi
    echo "$version"
}

# Download and install
install_binary() {
    local version="$1"
    local asset_name="${BINARY_NAME}-${os}-${arch}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"

    info "Downloading ${BINARY_NAME} ${version} (${os}/${arch})..."

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    if ! curl -fsSL -o "${tmp_dir}/${asset_name}" "$download_url"; then
        error "Download failed. Check if release assets exist for ${version}"
        exit 1
    fi

    info "Extracting..."
    tar -xzf "${tmp_dir}/${asset_name}" -C "$tmp_dir"

    info "Installing to ${INSTALL_DIR}/..."
    mkdir -p "$INSTALL_DIR"
    install -m 755 "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

    info "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Check if already installed
check_installed() {
    if command -v "$BINARY_NAME" &>/dev/null; then
        local current
        current=$("$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        warn "${BINARY_NAME} is already installed (${current})"
        read -r -p "Overwrite? [y/N] " confirm
        if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
            info "Aborted."
            exit 0
        fi
    fi
}

# Ensure PATH includes install dir
check_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH"
            warn "Add this to your shell profile:"
            warn ""
            warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            warn ""
            ;;
    esac
}

main() {
    detect_os
    detect_arch
    check_installed

    local version
    version=$(get_latest_version)
    install_binary "$version"
    check_path

    info "Done! Run '${BINARY_NAME}' to start."
}

main "$@"
