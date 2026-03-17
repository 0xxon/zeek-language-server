#!/usr/bin/env bash
# Build the Zeek LSP WASM module and output to site/pkg/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SITE_DIR="$(dirname "$SCRIPT_DIR")/site"

# Clang-19 is needed to compile tree-sitter C code for wasm32.
export CC_wasm32_unknown_unknown=clang-19
export AR_wasm32_unknown_unknown=llvm-ar-19
EMSCRIPTEN_SYSROOT="${EMSCRIPTEN_SYSROOT_INCLUDE:-/usr/share/emscripten/cache/sysroot/include}"
export CFLAGS_wasm32_unknown_unknown="-I${EMSCRIPTEN_SYSROOT} -matomics -mbulk-memory -mmutable-globals"

# tree-sitter's build.rs expects these env vars when targeting wasm32.
# It tries to compile stub C files from the path — we provide empty ones
# since our wasm_libc.rs handles everything at runtime.
TS_WASM_STUBS="$(mktemp -d)"
touch "$TS_WASM_STUBS/stdio.c" "$TS_WASM_STUBS/stdlib.c" "$TS_WASM_STUBS/string.c"
export DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS="$TS_WASM_STUBS"
export DEP_TREE_SITTER_LANGUAGE_WASM_SRC="$TS_WASM_STUBS"
trap 'rm -rf "$TS_WASM_STUBS"' EXIT

echo "Building Zeek LSP WASM..."

wasm-pack build \
  --target web \
  --out-dir "$SITE_DIR/pkg" \
  --release \
  "$SCRIPT_DIR/crates/zeek-lsp-web"

echo "Done. Output at: $SITE_DIR/pkg/"
echo "To serve: cd $SITE_DIR && python3 serve.py"
