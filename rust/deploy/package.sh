#!/usr/bin/env bash
# Builds the release folder for a server: the program plus the files it
# reads (templates, translations, images, CSS, JavaScript, public/), laid out
# like the repository. Copy the resulting .tar.gz to the server and unpack it
# into /opt/pila-derflice/releases/<version> (see SWITCH-OVER.md).
#
#   rust/deploy/package.sh            -> rust/dist/pila-derflice-<commit>.tar.gz
#
# Build on the same kind of system as the server (e.g. Debian/Ubuntu x86_64):
# the program only needs the C library, nothing else installed.
set -euo pipefail

crate="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$crate/.." && pwd)"
version="$(git -C "$repo" rev-parse --short HEAD)"
name="pila-derflice-$version"
out="$crate/dist/$name"

cargo build --release --manifest-path "$crate/Cargo.toml"

rm -rf "$out"
mkdir -p "$out/rust" "$out/config" "$out/app"
cp "$crate/target/release/piladerflice" "$out/"
cp -r "$crate/templates" "$crate/vendor" "$out/rust/"
cp -r "$repo/config/locales" "$out/config/"
cp -r "$repo/app/assets" "$repo/app/javascript" "$out/app/"
cp -r "$repo/public" "$out/"
cp -r "$crate/deploy" "$out/"
echo "$version" > "$out/VERSION"

tar -czf "$crate/dist/$name.tar.gz" -C "$crate/dist" "$name"
echo "$crate/dist/$name.tar.gz ($(du -h "$crate/dist/$name.tar.gz" | cut -f1))"
