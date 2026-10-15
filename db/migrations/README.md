# Database migrations

Every change to the tables is a file here, applied in order. `20261015120000_create_tables.sql` creates them all; later files change them.

1. Add a file `<timestamp>_<what>.sql`, e.g. `20261101120000_add_note_to_orders.sql`, with plain SQLite statements:

   ```sql
   ALTER TABLE orders ADD COLUMN note TEXT;
   ```

2. Change the code that uses it, and `tests/fixtures/*.sql` if needed.
3. Locally: `cargo run -- migrate`. On the server, after a backup: `piladerflice migrate`, then restart.

The migrations are built into the program. Each runs once, in a transaction, and is recorded (with a checksum) in the `_sqlx_migrations` table. The server refuses to start while one is waiting. Never edit a migration that already ran somewhere; add a new one.

Tests build their database from these files, so a migration is tested as soon as it is added.
