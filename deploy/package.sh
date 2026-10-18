#!/usr/bin/env bash
# Builds the release folder for a server: the program, the islands'
# WebAssembly (pkg/) and the files the program reads (translations, images,
# CSS, public/, the database schema and migrations), laid out like the
# repository. Copy the resulting .tar.gz to the server and unpack it into
# /opt/pila-derflice/releases/<version> (see LAUNCH.md).
#
#   deploy/package.sh            -> dist/pila-derflice-<commit>.tar.gz
#
# Needs cargo-leptos (`cargo install cargo-leptos --locked`) and the
# WebAssembly target (`rustup target add wasm32-unknown-unknown`).
# Build on the same kind of system as the server (e.g. Debian/Ubuntu x86_64):
# the program only needs the C library, nothing else installed.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(git -C "$root" rev-parse --short HEAD)"
name="pila-derflice-$version"
out="$root/dist/$name"

(cd "$root" && cargo leptos build --release)

rm -rf "$out"
mkdir -p "$out/pkg"
cp "$root/target/release/piladerflice" "$out/"
cp "$root/target/site/pkg/piladerflice.js" "$root/target/site/pkg/piladerflice.wasm" "$out/pkg/"
cp -r "$root/locales" "$root/assets" "$root/public" "$root/db" "$root/deploy" "$out/"
echo "$version" > "$out/VERSION"

tar -czf "$root/dist/$name.tar.gz" -C "$root/dist" "$name"
echo "$root/dist/$name.tar.gz ($(du -h "$root/dist/$name.tar.gz" | cut -f1))"
