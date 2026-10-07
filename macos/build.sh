#!/bin/sh
# Builds "Odomouse.app" without Xcode: only the Command Line Tools
# (swiftc, already there if Homebrew is installed) and Rust are needed.
#
#   sh macos/build.sh                      # this Mac's architecture
#   ARCHS="arm64 x86_64" sh macos/build.sh # universal (Apple Silicon + Intel)
#
# Result: macos/dist/Odomouse.app
set -e
cd "$(dirname "$0")"
here="$(pwd)"
root="$(cd .. && pwd)"
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
export MACOSX_DEPLOYMENT_TARGET=11.0

# ---- tools
if ! xcode-select -p >/dev/null 2>&1; then
  echo "Command Line Tools kerak. O'rnatish oynasi ochiladi; tugagach, buyruqni qayta ishga tushiring."
  xcode-select --install || true
  exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust o'rnatilmoqda (bir marta)..."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
fi

archs="${ARCHS:-$(uname -m)}"
build="$here/build"
rm -rf "$build" && mkdir -p "$build"

# ---- Rust core, one static library per architecture
for arch in $archs; do
  case "$arch" in
    arm64) target=aarch64-apple-darwin ;;
    x86_64) target=x86_64-apple-darwin ;;
    *) echo "Noma'lum arxitektura: $arch"; exit 1 ;;
  esac
  if command -v rustup >/dev/null 2>&1; then
    rustup target list --installed | grep -q "^$target\$" || rustup target add "$target"
  fi
  echo "Yadro ($arch)..."
  cargo build --quiet --manifest-path "$root/core/Cargo.toml" --release --target "$target"
  mkdir -p "$build/$arch"
  cp "$root/core/target/$target/release/libodomouse_core.a" "$build/$arch/"

  echo "Ilova ($arch)..."
  swiftc -O -whole-module-optimization \
    -target "$arch-apple-macos11.0" \
    -module-name Odomouse \
    -import-objc-header Odomouse/Bridging-Header.h \
    -I "$root/core/include" \
    -L "$build/$arch" -lodomouse_core \
    -o "$build/$arch/Odomouse" \
    Odomouse/*.swift
done

# ---- bundle
app="$build/Odomouse.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources/core"
bins=""
for arch in $archs; do bins="$bins $build/$arch/Odomouse"; done
# shellcheck disable=SC2086
lipo -create $bins -output "$app/Contents/MacOS/Odomouse"
cp Odomouse/Info.plist "$app/Contents/Info.plist"
cp Resources/AppIcon.icns Resources/TrayIcon.png Resources/TrayIcon@2x.png "$app/Contents/Resources/"
cp -R "$root/src/ui" "$root/src/assets" "$app/Contents/Resources/"
cp "$root/src/core/units.js" "$root/src/core/keyboard.js" "$root/src/core/shortcuts.js" "$root/src/core/fun.js" "$app/Contents/Resources/core/"
codesign --force --sign - "$app"

mkdir -p dist
rm -rf "dist/Odomouse.app"
cp -R "$app" "dist/"
echo
echo "Tayyor: $here/dist/Odomouse.app ($(lipo -archs "dist/Odomouse.app/Contents/MacOS/Odomouse"))"
