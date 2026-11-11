# Launching piladerflice.cz

How the site gets onto its own server, step by step. Nothing here needs a Mac
or Linux: deploys run on GitHub, so from Windows you only need a browser and
PowerShell (for one SSH key).

**How it fits together**

- One small Linux server (OVHcloud VPS-1, ~110 Kč a month) runs the site in a
  Docker container.
- **Deploy:** GitHub → Actions → **Deploy** → Run workflow.
  - It runs the tests first. If they pass, Kamal builds the image, uploads it to GitHub's registry, and swaps it in on the server with no downtime.
  - It also gets and renews the HTTPS certificate (Let's Encrypt).
- **Data:** the database (orders, catalogue, customers) and the uploaded photos
  live in a Docker volume on the server called `piladerflice_storage`.
  - Deploys never touch it.
  - Every night at 03:15 a copy goes to `/var/backups/piladerflice` on the server, and the last 14 days are kept.
- **Config:** `config/deploy.yml` (Kamal), `Dockerfile`, `.github/workflows/deploy.yml`, `config/backup/backup.sh`.

---

## 1. Server

1. Create an account at **European** [OVHcloud](https://www.ovhcloud.com/cs/vps/) and order a **VPS-1** (2 vCores, 4 GB RAM, 40 GB NVMe, daily backup included). Hetzner's CX23 works the same way when it's in stock.
   Use the European site (`ovhcloud.com/cs/` or `/en-ie/`), not `ovhcloud.com/en/` with dollar prices: that's OVHcloud US, a separate company with US data centres only, and its accounts can't order European servers.
2. Make an SSH key for deploys. In PowerShell on your computer:
   ```powershell
   ssh-keygen -t ed25519 -C "piladerflice deploy" -f $HOME\.ssh\piladerflice_deploy
   ```
   Press Enter twice; leave the passphrase empty, because GitHub uses this key unattended.
   It makes two files:
   - `piladerflice_deploy.pub` is the **public** half. It goes on the server.
   - `piladerflice_deploy` is the **private** half. It goes into GitHub secrets and nowhere else.
3. Configure the server:
   - **Image:** Ubuntu 24.04.
   - **Location:** Frankfurt (Germany), closest to us; Warsaw or Strasbourg also fine.
   - **SSH key:** paste the contents of `piladerflice_deploy.pub`.
   - **Backups:** VPS-1 includes a daily backup of the whole server. It's the
     off-server copy if the machine itself dies, on top of our own nightly
     backups.
   - **Firewall** (optional, in the OVH panel): allow inbound ports 22, 80 and 443 only.
   - **Type:** if you ever pick another plan, it must be x86 (Intel/AMD), not ARM (e.g. Hetzner CAX), because the image is built for x86.
4. Note the server's **IPv4 address**. On OVH's Ubuntu you log in as
   **`ubuntu`**, not `root`. That's why `SSH_USER` is set to `ubuntu` below.

Docker doesn't need installing on the server. Kamal does that on the first deploy.

## 2. Settings on GitHub

Go to the repository → **Settings → Secrets and variables → Actions**.

**Variables tab** (not secret):

| Name | Value |
|---|---|
| `SERVER_IP` | the server's IPv4, e.g. `116.203.10.20` |
| `SSH_USER` | `ubuntu` on OVH (leave unset for servers that log in as `root`) |
| `APP_HOSTS` | **for testing:** `116-203-10-20.sslip.io` (the IP with dashes, see step 4); **after the DNS switch:** `piladerflice.cz,www.piladerflice.cz` |
| `SMTP_ADDRESS` | the mail provider's SMTP server (see step 3) |
| `SMTP_PORT` | usually `587` |
| `MAIL_FROM_ADDRESS` | e.g. `objednavky@piladerflice.cz` |

**Secrets tab:**

| Name | Value |
|---|---|
| `SSH_PRIVATE_KEY` | the whole contents of `piladerflice_deploy`, the file **without** `.pub` |
| `ADMIN_EMAIL` / `ADMIN_PASSWORD` | the admin login, for the **Admin account** workflow |
| `SECRET_KEY_BASE` | a long random string: run `ruby bin/rails secret` in the project and paste the output. Set it once and never change it: changing it logs everyone out and breaks password-reset links. |
| `SMTP_USERNAME` / `SMTP_PASSWORD` | the mail account the site sends from |
| `ORDER_NOTIFICATION_EMAIL` | where new orders and inquiries go, e.g. Štěpán's e-mail |
| `ORS_API_KEY` | the openrouteservice key (delivery prices, address suggestions) |

The registry password needs no setting: the workflow uses GitHub's own token.

## 3. E-mail (decide before launch)

The site sends e-mails for:
- order confirmations to customers
- new orders and inquiries to the shop
- password resets

They're sent **from** `MAIL_FROM_ADDRESS`. To keep them out of spam folders, that address's domain has to allow the provider that sends them. That takes SPF and DKIM records in the DNS, which the provider tells you to add.

- **Easiest:** a transactional e-mail service such as Brevo, Mailjet or Postmark.
  - Add the domain there and copy the DNS records they show into Cloudflare.
  - Use their SMTP details for the settings above.
- **Gmail's SMTP** only works for sending from that Gmail address itself, not from `@piladerflice.cz`.

Until SMTP is set, the site works but sends no e-mails.

## 4. First deploy (test run, the old site keeps running)

1. Set `APP_HOSTS` to the server IP with dots replaced by dashes plus
   `.sslip.io`, e.g. `116-203-10-20.sslip.io`. That name points at the server
   without any DNS changes, and Let's Encrypt gives it a real certificate.
2. **Actions → Deploy → Run workflow.** The first run takes ~10 minutes:
   - It installs Docker on the server.
   - It starts the nightly backup job.
   - It deploys the site.
3. Open `https://116-203-10-20.sslip.io`.
   - On first start, the empty database is filled from the seeds: catalogue, price list, promos, FAQ.
   - The seeds refuse to run again on a database that already has content, so they can never wipe the live shop.
4. Create the admin account:
   - Add the secrets `ADMIN_EMAIL` and `ADMIN_PASSWORD`.
   - Run **Actions → Admin account → Run workflow**. Running it again later resets that admin's password.
5. Log in at `/admin` and check the catalogue, prices and photos.
6. Click through everything on a phone and a computer: the e-shop, the cart, a test order (it should e-mail you), the contact form, `/at`.

## 5. Going live (DNS switch)

This needs Štěpán to be the domain's owner ("Držitel") at WEDOS first.

1. Create a free [Cloudflare](https://dash.cloudflare.com) account → **Add a domain** → `piladerflice.cz` → Free plan.
   Cloudflare copies the domain's current DNS records.
   - Check the list against the old DNS: keep the **MX** records and any **TXT** records exactly as they are, so e-mail keeps working.
   - Add the SPF/DKIM records from the mail provider (step 3).
2. In Cloudflare, set the website records (**proxy off**, grey cloud, at least for the launch):
   - `A` `piladerflice.cz` → server IP
   - `CNAME` `www` → `piladerflice.cz`
3. In GitHub, change `APP_HOSTS` to `piladerflice.cz,www.piladerflice.cz` and run **Deploy** again.
4. In the WEDOS account, set the domain's nameservers to the two Cloudflare shows. It takes from minutes to a few hours.
5. When `https://piladerflice.cz` shows the new site, check:
   - `https://www.piladerflice.cz/obchod/` redirects to the new e-shop.
   - The old WordPress addresses redirect too. They're all listed in `config/routes/legacy.rb`, and Google keeps its rankings this way.
   - A test order goes through.
6. In [Google Search Console](https://search.google.com/search-console):
   - Add `piladerflice.cz`.
   - Submit `https://piladerflice.cz/sitemap.xml`.

## Everyday

- **Deploy a change:** merge it into `main`, then **Actions → Deploy → Run workflow**.
- **Restore a backup:** each night is a folder in `/var/backups/piladerflice/`
  holding `production.sqlite3.gz` (the database) and `uploads.tar.gz` (photos).
  1. Stop the app: `kamal app stop`.
  2. Unpack both into the `piladerflice_storage` volume:
     `docker volume inspect piladerflice_storage` shows the volume's folder on the server.
  3. Start the app again: `kamal app boot`.
- **Copy a backup to your computer:**
  `scp -i ~/.ssh/piladerflice_deploy -r ubuntu@SERVER_IP:/var/backups/piladerflice/<date> .`

### Running Kamal commands

Deploys and the admin account go through GitHub. For anything else (logs, the
console, a backup on demand) you run Kamal by hand. On Windows, use **WSL** (Ubuntu)
or simply SSH into the server and use plain `docker` there.

Kamal needs the same variables as the workflow. In the project folder, create
a `.env` file. It's ignored by git, so never commit it. Put these in it:
- `SERVER_IP=...`
- `APP_HOSTS=...`
- `KAMAL_REGISTRY_PASSWORD=` a GitHub token with `read:packages` and `write:packages`
- the other secrets

The private key must be loaded: `ssh-add ~/.ssh/piladerflice_deploy`.

| Command | Does |
|---|---|
| `bundle exec kamal logs` | live log of the site |
| `bundle exec kamal console` | Rails console on the live database (careful) |
| `bundle exec kamal dbconsole` | SQLite prompt on the live database |
| `bundle exec kamal backup-now` | take a backup right now |
| `bundle exec kamal accessory logs backup` | the backup job's output |
| `bundle exec kamal app exec "ruby bin/rails admin:create_user" -e ADMIN_EMAIL:… ADMIN_PASSWORD:…` | create or reset an admin |

Never run `kamal remove` or delete the `piladerflice_storage` volume: that is the shop's data.
