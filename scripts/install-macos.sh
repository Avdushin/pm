#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${1:-/usr/local/bin}"
BINARY_URL="https://github.com/Avdushin/pm/releases/latest/download/pm-macos-amd64"

echo "==> Installing pm for macOS"
echo "Install dir: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

echo "==> Downloading binary from:"
echo "    $BINARY_URL"
curl -fL "$BINARY_URL" -o "$INSTALL_DIR/pm"

chmod +x "$INSTALL_DIR/pm"
echo "✅ pm installed to $INSTALL_DIR/pm"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "✅ $INSTALL_DIR is already in PATH"
    ;;
  *)
    echo "⚠️  $INSTALL_DIR is not in PATH."
    echo "   Add this to your shell config, for example:"
    echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac
