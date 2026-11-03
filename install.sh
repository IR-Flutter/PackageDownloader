#!/usr/bin/env bash
set -euo pipefail

REPO="IR-Flutter/PackageDownloader"
BIN_NAME="pdown"
ALIAS_NAME="irflutter"

echo "================================================================"
echo "🚀 Installing PackageDownloader 2.0 (pdown)..."
echo "================================================================"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)
    case "$ARCH" in
      x86_64)
        TARGET="linux-x64"
        ;;
      *)
        echo "❌ Unsupported Linux architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  Darwin*)
    case "$ARCH" in
      arm64)
        TARGET="macos-arm64"
        ;;
      x86_64)
        TARGET="macos-x64"
        ;;
      *)
        echo "❌ Unsupported macOS architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "❌ Unsupported Operating System: $OS"
    exit 1
    ;;
esac

echo "🔍 Fetching latest release information..."
LATEST_TAG=$(curl -sSL -H "Accept: application/vnd.github.v3+json" "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
if [ -z "$LATEST_TAG" ]; then
  LATEST_TAG="v2.0.0"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$BIN_NAME-$TARGET"
CHECKSUM_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/checksums.txt"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "📥 Downloading $BIN_NAME ($TARGET) from $DOWNLOAD_URL..."
curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/$BIN_NAME"

echo "🔒 Verifying SHA-256 cryptographic checksum..."
if curl -sSL -f "$CHECKSUM_URL" -o "$TMP_DIR/checksums.txt"; then
  EXPECTED_SHA=$(grep "$BIN_NAME-$TARGET" "$TMP_DIR/checksums.txt" | awk '{print $1}')
  if [ -n "$EXPECTED_SHA" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
      ACTUAL_SHA=$(sha256sum "$TMP_DIR/$BIN_NAME" | awk '{print $1}')
    else
      ACTUAL_SHA=$(shasum -a 256 "$TMP_DIR/$BIN_NAME" | awk '{print $1}')
    fi

    if [ "$EXPECTED_SHA" != "$ACTUAL_SHA" ]; then
      echo "❌ Checksum verification failed!"
      echo "   Expected: $EXPECTED_SHA"
      echo "   Actual:   $ACTUAL_SHA"
      exit 1
    fi
    echo "✅ Checksum verified: $ACTUAL_SHA"
  fi
fi

chmod +x "$TMP_DIR/$BIN_NAME"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

# Create irflutter backwards-compatibility alias symlink
ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/$ALIAS_NAME"

echo ""
echo "🎉 PackageDownloader 2.0 successfully installed to $INSTALL_DIR/$BIN_NAME"
echo "   (Alias created: $INSTALL_DIR/$ALIAS_NAME)"
echo ""
echo "Get started by running:"
echo "  pdown list"
echo "  pdown doctor"
echo "  pdown use pub-azs"
