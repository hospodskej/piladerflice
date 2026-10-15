#!/usr/bin/env bash
# Builds the release folder for a server: the program plus the files it
# reads (templates, translations, images, CSS, JavaScript, public/, the
# database schema and migrations), laid out like the repository. Copy the resulting .tar.gz to the server and unpack it
# into /opt/pila-derflice/releases/<version> (see SWITCH-OVER.md).
#
#   deploy/package.sh            -> dist/pila-derflice-<commit>.tar.gz
#
# Build on the same kind of system as the server (e.g. Debian/Ubuntu x86_64):
# the program only needs the C library, nothing else installed.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(git -C "$root" rev-parse --short HEAD)"
name="pila-derflice-$version"
out="$root/dist/$name"

cargo build --release --manifest-path "$root/Cargo.toml"

rm -rf "$out"
mkdir -p "$out"
cp "$root/target/release/piladerflice" "$out/"
cp -r "$root/templates" "$root/locales" "$root/assets" "$root/javascript" "$root/vendor" \
      "$root/public" "$root/db" "$root/deploy" "$out/"
echo "$version" > "$out/VERSION"

tar -czf "$root/dist/$name.tar.gz" -C "$root/dist" "$name"
echo "$root/dist/$name.tar.gz ($(du -h "$root/dist/$name.tar.gz" | cut -f1))"
