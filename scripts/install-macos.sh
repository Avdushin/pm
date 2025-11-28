#!/usr/bin/env bash
set -euo pipefail

echo "==> Building pm (release)..."
cargo build --release

INSTALL_DIR="${1:-/usr/local/bin}"

echo "==> Installing to: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
cp target/release/pm "$INSTALL_DIR/pm"

echo "✅ pm installed to $INSTALL_DIR"

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
