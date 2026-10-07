#!/bin/sh
# Builds linux/dist/odomouse_<version>_<arch>.deb for Ubuntu, Mint, Debian,
# Pop!_OS... (double-click it, or: sudo apt install ./file.deb)
set -e
cd "$(dirname "$0")"
cargo build --release
arch="$(uname -m)"
version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -n1)"
rm -rf dist && mkdir -p dist

# ---- shared payload: the program and the web UI it serves
stage_web() {
  mkdir -p "$1/web/core"
  cp -r ../src/ui ../src/assets "$1/web/"
  cp ../src/core/units.js ../src/core/keyboard.js ../src/core/shortcuts.js ../src/core/fun.js "$1/web/core/"
}

# ---- .deb
case "$arch" in
  x86_64) debarch=amd64 ;;
  aarch64) debarch=arm64 ;;
  *) debarch="$arch" ;;
esac
pkg="dist/deb"
lib="$pkg/usr/lib/odomouse"
mkdir -p "$lib" "$pkg/usr/bin" "$pkg/usr/share/applications" \
  "$pkg/usr/share/icons/hicolor/256x256/apps" "$pkg/usr/share/doc/odomouse" "$pkg/DEBIAN"
install -m 0755 target/release/odomouse "$lib/odomouse"
stage_web "$lib"
ln -s ../lib/odomouse/odomouse "$pkg/usr/bin/odomouse"
install -m 0644 assets/odomouse.png "$pkg/usr/share/icons/hicolor/256x256/apps/odomouse.png"
cat > "$pkg/usr/share/applications/odomouse.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Odomouse
Comment=Sichqoncha masofasi, klaviatura va kliklar statistikasi
Exec=odomouse
Icon=odomouse
Categories=Utility;
Keywords=mouse;keyboard;statistics;sichqoncha;klaviatura;
Terminal=false
StartupNotify=false
DESKTOP
cat > "$pkg/usr/share/doc/odomouse/copyright" <<'COPYRIGHT'
Odomouse
All statistics stay on this computer (~/.local/share/odomouse/).
COPYRIGHT

cat > "$pkg/DEBIAN/control" <<CONTROL
Package: odomouse
Version: $version
Architecture: $debarch
Maintainer: Odomouse <noreply@odomouse.app>
Section: utils
Priority: optional
Depends: libc6 (>= 2.35)
Conflicts: mishka-tracker
Replaces: mishka-tracker
Provides: mishka-tracker
Recommends: libx11-6, libxtst6, libxrandr2, libdbus-1-3, xdg-utils
Suggests: chromium | google-chrome-stable | chromium-browser
Installed-Size: $(du -sk --exclude=DEBIAN "$pkg" | cut -f1)
Homepage: https://github.com/FoziljonSAP/odomouse
Description: mouse distance, keystrokes and clicks statistics
 A tiny tray app that counts how far the mouse travels, keys, letters,
 clicks, scrolling, active time and typing speed. Statistics open in the
 browser and never leave the computer.
CONTROL

cat > "$pkg/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q /usr/share/applications || true
  command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
exit 0
POSTINST
cat > "$pkg/DEBIAN/prerm" <<'PRERM'
#!/bin/sh
set -e
# stop running copies so the files can be replaced or removed (each saves on exit)
if [ "$1" = "remove" ] || [ "$1" = "upgrade" ]; then
  if pkill -TERM -x odomouse 2>/dev/null; then
    i=0
    while [ $i -lt 30 ] && pgrep -x odomouse >/dev/null 2>&1; do sleep 0.1; i=$((i + 1)); done
  fi
fi
exit 0
PRERM
cat > "$pkg/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q /usr/share/applications || true
fi
exit 0
POSTRM
chmod 0755 "$pkg/DEBIAN/postinst" "$pkg/DEBIAN/prerm" "$pkg/DEBIAN/postrm"
find "$pkg" -type d -exec chmod 0755 {} +
find "$pkg/usr" -type f ! -path "$lib/odomouse" -exec chmod 0644 {} +

deb="dist/odomouse_${version}_${debarch}.deb"
dpkg-deb --root-owner-group -Zxz --build "$pkg" "$deb" >/dev/null
rm -rf "$pkg"
echo "$deb"
