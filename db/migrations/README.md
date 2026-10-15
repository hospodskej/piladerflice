# Database migrations

Changes to the tables since the Rails app (whose last migration, 20261014100000, is `db/schema.sql`).

1. Add a file `<timestamp>_<what>.sql`, e.g. `20261101120000_add_note_to_orders.sql`, with plain SQLite statements:

   ```sql
   ALTER TABLE orders ADD COLUMN note varchar;
   ```

2. Change the code that uses it, and `tests/fixtures/*.sql` if needed.
3. Locally: `cargo run -- migrate`. On the server, after a backup: `piladerflice migrate`, then restart.

Each file runs once, in its own transaction, and is recorded in the `schema_migrations` table. The server refuses to start while one is waiting. Never edit a migration that already ran somewhere; add a new one.

Tests build their database from `db/schema.sql` plus these files, so a migration is tested as soon as it is added.
