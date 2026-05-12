#!/usr/bin/env bash
set -e

ARCH=$(uname -m)
RUST_TARGET="${ARCH}-unknown-linux-gnu"

rm -rf AppDir
mkdir -p AppDir/usr/lib64
export LD_LIBRARY_PATH="target/release:$LD_LIBRARY_PATH"
export LINUXDEPLOY_EXCLUDED_LIBRARIES="\
  libxkbcommon.so*;libxkbcommon-x11.so*;\
  libwayland-client.so*;libwayland-cursor.so*;libwayland-egl.so*;\
  libglib-2.0.so*;libgobject-2.0.so*;libgio-2.0.so*;libgmodule-2.0.so*;libgthread-2.0.so*;\
  libgtk-3.so*;libgdk-3.so*;\
  libwebkit2gtk*.so*;libjavascriptcoregtk*.so*;\
  libicudata.so*;libicui18n.so*;libicuuc.so*;\
  libcairo.so*;libcairo-gobject.so*;\
  libpango-1.0.so*;libpangocairo-1.0.so*;libpangoft2-1.0.so*;\
  libatk-1.0.so*;libatk-bridge-2.0.so*;libatspi.so*;\
  libepoxy.so*;\
  libharfbuzz.so*;libfontconfig.so*;libfreetype.so*;\
  libpixman-1.so*;\
  libdbus-1.so*;\
  libsecret-1.so*;\
  libgcrypt.so*;libgpg-error.so*;\
  libffi.so*;\
  libpcre2-8.so*;libpcre2-16.so*;libpcre2-32.so*"
export LINUXDEPLOY_EXCLUDED_LIBRARIES="${LINUXDEPLOY_EXCLUDED_LIBRARIES//[$'\n ']/}"

mkdir -p AppDir/apprun-hooks
cp scripts/apprun-hooks/gdk-x11.sh AppDir/apprun-hooks/gdk-x11.sh
chmod +x AppDir/apprun-hooks/gdk-x11.sh

NO_STRIP=1 APPIMAGE_EXTRACT_AND_RUN=1 linuxdeploy --appimage-extract-and-run --appdir AppDir -l target/release/libSDL3.so.0 -e target/release/SISR -d sisr.desktop -i docs/SISR.svg --output appimage
mkdir -p dist/appimage
mv SISR-${ARCH}.AppImage dist/appimage/
