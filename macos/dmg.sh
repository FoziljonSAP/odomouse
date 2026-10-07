#!/bin/sh
# Packs macos/dist/Odomouse.app (from build.sh) into
# macos/dist/Odomouse.dmg: the app next to an Applications shortcut,
# so installing is one drag. Run after build.sh, on a Mac.
set -e
cd "$(dirname "$0")/dist"
app="Odomouse.app"
[ -d "$app" ] || { echo "Avval: sh macos/build.sh"; exit 1; }

stage="$(mktemp -d)/Odomouse"
mkdir -p "$stage"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"

rm -f Odomouse.dmg
# hdiutil now and then fails with "Resource busy" on CI machines: retry
i=0
until hdiutil create -quiet -volname "Odomouse" -srcfolder "$stage" -fs HFS+ \
    -format UDZO -imagekey zlib-level=9 -ov Odomouse.dmg; do
  i=$((i + 1))
  [ $i -ge 5 ] && exit 1
  sleep 3
done
rm -rf "$(dirname "$stage")"
echo "Tayyor: $(pwd)/Odomouse.dmg"
