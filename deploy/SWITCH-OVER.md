# Switching the live site from Rails to Rust

The Rust version shows the same pages, takes the same orders and works on the same database and image folder as the Rails app. Switching is mostly moving the data to the new server and pointing the domain at it. Going back is the same steps in reverse, because Rails can keep using the data Rust has written.

## What visitors notice

- Everyone is signed out once, and carts are emptied (the cookies have new names). Accounts and passwords stay.
- Old image addresses (`/rails/active_storage/...`, from Google Images, shared links, old e-mails) are sent on to the same pictures.
- Nothing else: every page has the same HTML as in Rails.

## Before the switch

1. **Merge the open Rails fixes** (they change shared translations) and build the release from that commit.
2. **The live database must have the Rails app's last migration** (20261014100000, what `db/schema.sql` describes); the program refuses an older one and tells you so. Migrations added since (`db/migrations/`) are applied with `piladerflice migrate` (below).
3. **Find the live data** on the current server: the Rails production database (`DATABASE_URL`) and its `storage/` folder (uploaded photos, profile pictures).
4. **Try it on a copy first**: set up the new server as below with a copy of the data, and go through the checklist at the end. Nothing changes for visitors until the domain points at it.
5. **Lower the DNS TTL** of piladerflice.cz to 5 minutes a day ahead, so the switch spreads quickly (and a switch back too).
6. **New secrets**: generate a new `SECRET_KEY_BASE` (`openssl rand -hex 64`). If the openrouteservice key was ever shared outside the server, create a new one.

## Setting up the server (once)

On Debian or Ubuntu, as root:

```sh
apt install caddy
adduser --system --group --home /var/lib/pila-derflice pila
mkdir -p /opt/pila-derflice/releases /var/lib/pila-derflice/storage /var/backups/pila-derflice
chown -R pila:pila /var/lib/pila-derflice /var/backups/pila-derflice

cp pila-derflice.env.example /etc/pila-derflice.env     # then fill it in
chown root:pila /etc/pila-derflice.env && chmod 640 /etc/pila-derflice.env

cp pila-derflice.service pila-derflice-backup.service pila-derflice-backup.timer /etc/systemd/system/
cp Caddyfile /etc/caddy/Caddyfile
systemctl daemon-reload
systemctl enable pila-derflice pila-derflice-backup.timer
```

(The files are in `deploy/`, also in every release folder.)

## Putting a release on the server

On your computer (or a build machine with the same system as the server):

```sh
deploy/package.sh                      # -> dist/pila-derflice-<commit>.tar.gz
scp dist/pila-derflice-<commit>.tar.gz server:/tmp/
```

On the server:

```sh
tar -xzf /tmp/pila-derflice-<commit>.tar.gz -C /opt/pila-derflice/releases
ln -sfn /opt/pila-derflice/releases/pila-derflice-<commit> /opt/pila-derflice/current
# If the release has new database migrations: back up, then apply them.
sudo -u pila sh -c 'set -a; . /etc/pila-derflice.env; set +a;
  /opt/pila-derflice/current/piladerflice backup /var/backups/pila-derflice/before-migrate-$(date +%s).sqlite3 &&
  /opt/pila-derflice/current/piladerflice migrate'
systemctl restart pila-derflice
journalctl -u pila-derflice -n 20                # "listening on http://127.0.0.1:4000 (production)"
```

To go back to the previous release, point `current` at its folder and restart. Releases are small; keep the last few.

## The switch

Plan 15 minutes in a quiet hour (orders can't be taken while the data moves).

1. **Stop the Rails app**, so nothing is written any more.
2. **Copy the database**: with Rails stopped, the `.sqlite3` file (and any `-wal` / `-shm` files next to it) to `/var/lib/pila-derflice/production.sqlite3` on the new server.
3. **Copy the images**: the Rails `storage/` folder's subfolders to `/var/lib/pila-derflice/storage/` (`rsync -a`), not the database files in it.
4. `chown -R pila:pila /var/lib/pila-derflice`
5. **Start Rust**: `systemctl start pila-derflice`, then `systemctl reload caddy`.
6. **Point the domain** (DNS A/AAAA records) at the new server. Caddy gets the HTTPS certificate by itself within a minute.
7. **Go through the checklist** below.

## Checklist after the switch

- [ ] Home page, e-shop, a product page, contact page, in Czech and at /at in German
- [ ] Product photos show; an old `/rails/active_storage/...` image link opens the photo
- [ ] Put something in the cart and place a test order: the confirmation page, the e-mail to the customer and the one to the shop arrive
- [ ] Delivery to an address shows a price by distance (openrouteservice works)
- [ ] Send a quote request from the contact page: the e-mail arrives
- [ ] Sign in to an existing customer account; the order history is there
- [ ] Sign in to /admin; the test order is listed; delete it
- [ ] Change a price in the admin and see it on the product page; upload a photo
- [ ] `https://piladerflice.cz/up` says OK; `http://` goes to `https://`
- [ ] The next morning: a backup in `/var/backups/pila-derflice` (and copy that folder off the server regularly)

## Going back

Rails reads everything Rust writes (same tables, same password hashes, same image storage; tested both ways), as long as no migration has changed the tables since (a new column Rails doesn't know is fine, a removed or renamed one isn't). To return:

1. Stop Rust: `systemctl stop pila-derflice`.
2. Copy `/var/lib/pila-derflice/production.sqlite3` and `storage/` back to the Rails server.
3. Start Rails and point the domain back.

## After the switch

- **Backups** run every night at 03:15 (database and photos, kept 30 days). Copy them somewhere else too: a backup on the same disk doesn't survive the disk.
- **Logs**: `journalctl -u pila-derflice`. Errors are logged with what failed.
- **Database changes** are SQL files in `db/migrations/` (see the README there), applied with `piladerflice migrate` after a backup. No Ruby needed.
- **Updates** of the program: package, copy, switch `current`, restart (a few seconds; requests in progress are finished first).
