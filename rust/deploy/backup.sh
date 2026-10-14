#!/usr/bin/env bash
# Daily backup: a consistent copy of the database and the uploaded images,
# kept for 30 days. Run by pila-derflice-backup.timer, as the pila user.
# Copy /var/backups/pila-derflice somewhere else too (another machine, a
# cloud drive): a backup on the same disk doesn't survive the disk.
set -euo pipefail

set -a; source /etc/pila-derflice.env; set +a
dest=/var/backups/pila-derflice
stamp="$(date +%Y-%m-%d)"
mkdir -p "$dest"

/opt/pila-derflice/current/piladerflice backup "$dest/production-$stamp.sqlite3"
tar -czf "$dest/storage-$stamp.tar.gz" -C "$(dirname "$STORAGE_DIR")" "$(basename "$STORAGE_DIR")"

find "$dest" -name 'production-*.sqlite3' -mtime +30 -delete
find "$dest" -name 'storage-*.tar.gz' -mtime +30 -delete
