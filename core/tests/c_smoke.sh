#!/bin/sh
# Build the release static library and run the C ABI smoke test.
set -e
cd "$(dirname "$0")/.."
cargo build --release --quiet
out="${TMPDIR:-/tmp}/odomouse-c-smoke"
rm -rf "$out" && mkdir -p "$out"
cc -std=c99 -Wall -Wextra -o "$out/smoke" tests/c_smoke.c target/release/libodomouse_core.a -lpthread -ldl -lm
"$out/smoke" "$out/data"
test -f "$out/data/history.json"
test -f "$out/data/settings.json"
