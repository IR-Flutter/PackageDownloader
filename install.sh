#!/bin/bash
set -e

REPO="IR-Flutter/PackageDownloader"
BIN_NAME="irflutter"

echo "🚀 Installing $BIN_NAME for Linux / macOS..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)
    TARGET="linux-x64"
    ;;
  Darwin*)
    if [ "$ARCH" = "arm64" ]; then
      TARGET="macos-arm64"
    else
      TARGET="macos-x64"
    fi
    ;;
  *)
    echo "❌ Unsupported OS: $OS"
    exit 1
    ;;
esac

LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$LATEST_RELEASE" ]; then
  LATEST_RELEASE="v1.0.0"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/$BIN_NAME-$TARGET"

echo "📥 Downloading $BIN_NAME ($TARGET) from $DOWNLOAD_URL ..."

TMP_DIR=$(mktemp -d)
curl -sL "$DOWNLOAD_URL" -o "$TMP_DIR/$BIN_NAME"
chmod +x "$TMP_DIR/$BIN_NAME"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
rm -rf "$TMP_DIR"

echo "✅ $BIN_NAME successfully installed to $INSTALL_DIR/$BIN_NAME"
echo ""
echo "Try running:"
echo "  $BIN_NAME list"
echo "  $BIN_NAME use pub-azs"
