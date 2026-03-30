#!/bin/sh
# Klar installer — https://kler.run
# Usage: curl -fsSL kler.run/install.sh | sh

set -e

REPO="klar-lang/klar"
INSTALL_DIR="${KLAR_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

info() { printf "%b>%b %s\n" "$GREEN" "$RESET" "$1"; }
error() { printf "%berror%b: %s\n" "$RED" "$RESET" "$1" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$OS" in
    Linux)  OS="linux" ;;
    Darwin) OS="macos" ;;
    *)      error "Unsupported OS: $OS" ;;
  esac

  case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *)             error "Unsupported architecture: $ARCH" ;;
  esac

  PLATFORM="${OS}-${ARCH}"
}

# Get latest release tag from GitHub (includes pre-releases)
get_latest_version() {
  # Try stable release first, then fall back to any release (including pre-release)
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  if [ -z "$VERSION" ]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases" 2>/dev/null | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  fi
  if [ -z "$VERSION" ]; then
    error "Could not determine latest version. Check https://github.com/${REPO}/releases"
  fi
}

# Download and install
install() {
  ARTIFACT="klar-${PLATFORM}"
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}"

  info "Detected platform: ${PLATFORM}"
  info "Installing klar ${VERSION}..."

  TMPDIR=$(mktemp -d)
  trap 'rm -rf "$TMPDIR"' EXIT

  curl -fsSL "$URL" -o "${TMPDIR}/klar" || error "Download failed. Binary may not exist for ${PLATFORM} yet."

  chmod +x "${TMPDIR}/klar"

  if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/klar" "${INSTALL_DIR}/klar"
  else
    info "Requesting sudo to install to ${INSTALL_DIR}..."
    sudo mv "${TMPDIR}/klar" "${INSTALL_DIR}/klar"
  fi

  printf "%b>%b %bklar %s%b installed to %s/klar\n" "$GREEN" "$RESET" "$BOLD" "$VERSION" "$RESET" "$INSTALL_DIR"
  printf "%b>%b Run %bklar --help%b to get started\n" "$GREEN" "$RESET" "$BOLD" "$RESET"
}

main() {
  printf "\n  %bKlar Installer%b\n" "$BOLD" "$RESET"
  printf "  The AI-first programming language\n\n"

  detect_platform
  get_latest_version
  install
}

main
