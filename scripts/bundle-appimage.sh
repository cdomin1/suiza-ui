#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
version=${1:-0.1.0-alpha.3}
[ "$(uname -s)" = Linux ] || { printf 'Build AppImages on Linux.\n' >&2; exit 1; }
[ "$(uname -m)" = x86_64 ] || { printf 'This package targets x86_64.\n' >&2; exit 1; }
scripts/bundle-linux.sh
appdir=dist/SUIZA-Canvas.AppDir
mkdir -p "$appdir/usr"
cp -a dist/suiza-linux/. "$appdir/usr/"
mkdir -p "$appdir/usr/share/metainfo"
cp packaging/io.github.cdomin1.SuizaCanvas.metainfo.xml "$appdir/usr/share/metainfo/suiza.appdata.xml"
cp LICENSE "$appdir/usr/share/licenses/suiza/LICENSE"
# linuxdeploy resolves and bundles the executable's shared-library dependencies.
APPIMAGE_EXTRACT_AND_RUN=1 "${LINUXDEPLOY:?Set LINUXDEPLOY to the official linuxdeploy AppImage}" --appdir "$appdir" --library /usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0 --executable "$appdir/usr/bin/suiza" --desktop-file packaging/suiza.desktop --icon-file assets/icons/256/suiza.png
# linuxdeploy creates AppRun as a symlink to the binary. Remove the link
# before writing our wrapper so the executable is never overwritten.
rm -f "$appdir/AppRun"
file "$appdir/usr/bin/suiza" | grep -q "ELF 64-bit"
cat > "$appdir/AppRun" <<'RUN'
#!/bin/sh
set -eu
appdir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export LD_LIBRARY_PATH="$appdir/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="$appdir/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "$appdir/usr/bin/suiza" "$@"
RUN
chmod +x "$appdir/AppRun"
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "${APPIMAGETOOL:?Set APPIMAGETOOL to the official appimagetool AppImage}" "$appdir" "dist/SUIZA-Canvas-${version}-x86_64.AppImage"
