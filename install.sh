#!/usr/bin/env bash
set -euo pipefail

REPO="pr4j3sh/hippo"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="hippo"
FORCE=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}==> $1${NC}"; }
warn()  { echo -e "${YELLOW}==> $1${NC}"; }
error() { echo -e "${RED}==> $1${NC}" >&2; }

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --force|-f) FORCE=1 ;;
    esac
done

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

# Get current installed version
get_current_version() {
    local current
    current=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null | awk '{print $2}' || echo "")
    echo "$current"
}

# Compare semver: returns 0 if equal, 1 if $1 > $2, 2 if $1 < $2
semver_compare() {
    local IFS='.'
    read -ra v1 <<< "$1"
    read -ra v2 <<< "$2"

    for i in 0 1 2; do
        local a="${v1[i]:-0}"
        local b="${v2[i]:-0}"
        if (( a > b )); then return 0; fi
        if (( a < b )); then return 1; fi
    done
    return 0
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

# Check if already installed and handle updates
check_installed() {
    if [ ! -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        return
    fi

    local current
    current=$(get_current_version)

    if [ -z "$current" ]; then
        warn "${BINARY_NAME} is already installed (version unknown)"
        if [ "$FORCE" -eq 0 ]; then
            read -r -p "Overwrite? [y/N] " confirm
            if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
                info "Aborted."
                exit 0
            fi
        fi
        return
    fi

    local latest
    latest=$(get_latest_version)

    # Strip leading 'v' if present
    local current_clean="${current#v}"
    local latest_clean="${latest#v}"

    if [ "$current_clean" = "$latest_clean" ]; then
        info "Already up to date (${current})."
        exit 0
    fi

    info "Updating ${BINARY_NAME} ${current} → ${latest}..."
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
