# Putting the site online

The site is one program plus a folder of files it reads (templates, translations, images). It keeps its data in one SQLite file and the uploaded photos in one folder, and runs behind Caddy, which handles HTTPS.

## Before launching

1. **A server**: any small Linux machine (Debian or Ubuntu here). 1 vCPU and 1 GB of RAM is plenty; the program needs about 30 MB.
2. **Secrets**: a new `SECRET_KEY_BASE` (`openssl rand -hex 64`), the SMTP login for sending e-mails, and an openrouteservice key for delivery prices (create a fresh one; never put it in git).
3. **Lower the DNS TTL** of piladerflice.cz to 5 minutes a day ahead, so the change of address spreads quickly.

## Setting up the server (once)

As root:

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
```

## The first start

```sh
# The database with the starting content (catalogue, prices, promos, texts) and its photos:
sudo -u pila sh -c 'set -a; . /etc/pila-derflice.env; set +a; /opt/pila-derflice/current/piladerflice setup'
# The admin account (ADMIN_EMAIL and ADMIN_PASSWORD, given just for this command):
sudo -u pila sh -c 'set -a; . /etc/pila-derflice.env; set +a;
  ADMIN_EMAIL=... ADMIN_PASSWORD=... /opt/pila-derflice/current/piladerflice admin'
systemctl start pila-derflice
systemctl reload caddy
journalctl -u pila-derflice -n 20      # "listening on http://127.0.0.1:4000 (production)"
```

Then point the domain (DNS A/AAAA records) at the server. Caddy gets the HTTPS certificate by itself within a minute.

## Checklist

- [ ] Home page, e-shop, a product page, contact page, in Czech and at /at in German
- [ ] Product photos show
- [ ] Put something in the cart and place a test order: the confirmation page, the e-mail to the customer and the one to the shop arrive
- [ ] Delivery to an address shows a price by distance (openrouteservice works)
- [ ] Send a quote request from the contact page: the e-mail arrives
- [ ] Register a customer account, sign out and in again
- [ ] Sign in to /admin; the test order is listed; delete it
- [ ] Change a price in the admin and see it on the product page; upload a photo
- [ ] `https://piladerflice.cz/up` says OK; `http://` goes to `https://`
- [ ] The next morning: a backup in `/var/backups/pila-derflice` (and copy that folder off the server regularly)

## Updates

```sh
# a new release, as above, then:
ln -sfn /opt/pila-derflice/releases/pila-derflice-<commit> /opt/pila-derflice/current
# If the release has new database migrations: back up, then apply them.
sudo -u pila sh -c 'set -a; . /etc/pila-derflice.env; set +a;
  /opt/pila-derflice/current/piladerflice backup /var/backups/pila-derflice/before-migrate-$(date +%s).sqlite3 &&
  /opt/pila-derflice/current/piladerflice migrate'
systemctl restart pila-derflice
```

A restart takes a few seconds; requests in progress are finished first. The program refuses to start while a migration is waiting, and says so in the log. To go back to the previous release, point `current` at its folder and restart (if that release is older than a migration you applied, restore the backup made before it). Keep the last few releases.

## Running it

- **Backups** run every night at 03:15 (database and photos, kept 30 days). Copy them somewhere else too: a backup on the same disk doesn't survive the disk.
- **Logs**: `journalctl -u pila-derflice`. Errors are logged with what failed.
- **Database changes** are SQL files in `db/migrations/` (see the README there), applied with `piladerflice migrate` after a backup.
