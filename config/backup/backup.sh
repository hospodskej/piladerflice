#!/bin/sh
# Nightly backup of the live site, run by the "backup" accessory (config/deploy.yml).
# /data is the app's storage volume, /backups is /var/backups/piladerflice on the server.
# Each night gets its own folder with the database and the uploaded photos;
# folders older than KEEP_DAYS days are deleted.
set -eu

KEEP_DAYS="${KEEP_DAYS:-14}"
dir="/backups/$(date +%Y-%m-%d_%H%M)"
mkdir -p "$dir"

# sqlite3's .backup takes a consistent copy even while the site is writing.
sqlite3 /data/production.db ".backup '$dir/production.db'"
gzip "$dir/production.db"

# Uploaded photos (everything in storage except the databases).
tar -czf "$dir/uploads.tar.gz" -C /data --exclude='*.db' --exclude='*.db-wal' --exclude='*.db-shm' .

find /backups -mindepth 1 -maxdepth 1 -type d -mtime +"$KEEP_DAYS" -exec rm -rf {} +
echo "$(date '+%F %T') backup ok: $dir ($(du -sh "$dir" | cut -f1))"
