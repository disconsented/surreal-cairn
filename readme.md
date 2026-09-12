# surreal-cairn

An up-only migration runner for [SurrealDB](https://surrealdb.com). You add a
file, it runs the file, it records that the file ran. Nothing ever comes back
off the pile, which is where the name comes from.

## This code is AI slop

Claude (Anthropic) wrote every line of this crate, including this readme and
the tests. No human wrote any of it. No human has read it line by line. The
tests pass against an in-memory instance and against RocksDB, which tells you
the tests pass and nothing else.

This thing runs SurrealQL against your database with no undo. Read the source
before you point it at data you care about, and take a backup first. There is
no support and no promise of a stable API. Two other SurrealDB migration
crates exist, `surrealdb-migrations` and `surrealdb-migrate`, and a human
wrote both of them. Use one of those if you want a dependency you can lean on.

## Use

```rust
use futures::StreamExt as _;
use surreal_cairn::{Migrator, Outcome};

let plan = Migrator::from_files("./migrations")?
    .with_table("_migrations")
    .ignore_checksum_changes(false)
    .validate()?
    .plan(&db)
    .await?;

println!("will apply {} migrations", plan.pending().len());

// The stream is not Unpin. Pin it before you poll it.
let mut stream = std::pin::pin!(plan.execute(&db));
while let Some(outcome) = stream.next().await {
    match outcome? {
        Outcome::Applied { id, duration } => println!("applied {id} in {duration:?}"),
        Outcome::Skipped { id, reason } => println!("skipped {id}: {reason:?}"),
    }
}
```

The type states stop you from running a step out of order:

```text
Migrator<Unvalidated>
    .with_table(..)                 builder methods live here only
    .ignore_checksum_changes(..)
    .validate()?          -> Migrator<Validated>
    .plan(&db).await?     -> Plan
    .execute(&db)         -> impl Stream<Item = Result<Outcome, Error>>
```

`validate()` needs no connection. It refuses duplicate IDs and empty files.
`plan()` reads the state table and sorts each migration into pending or
skipped. `execute()` runs the pending ones in order and stops at the first
error.

`Migrator::from_strings` takes the migrations from memory instead of from
disk, which is what the tests use.

## Migration files

`from_files` reads every `*.surql` file in the directory and sorts them by
filename, so the usual `<timestamp>_<name>.surql` shape works. The filename is
the migration ID. Renaming an applied file makes it look new, so do not rename
one.

The binary writes an empty stub for you:

```sh
surreal-cairn add_apps_table --dir migrations
# migrations/1789491600000_add_apps_table.surql
```

Build without the `cli` feature to drop `clap` and get the library alone.

## State table

Each applied migration becomes one row in `_migrations` (or whatever you pass
to `with_table`), holding the name, a checksum, the time it was applied, and
how long it took.

The checksum is BLAKE2b-256 over the file content, hex encoded. On the next
run the stored hash and the hash of the file on disk must match. A mismatch
means somebody edited a migration after it ran, and it fails with
`Error::ChecksumMismatch`. Call `.ignore_checksum_changes(true)` to log a
warning and continue instead.

## Known rough edges

The migration and its state row go into one transaction, so a failure leaves
no row behind. That guarantee is only as good as SurrealDB's transaction
handling for the statements you write, and schema statements are the ones to
watch.

There is no down migration and there will not be one.

## Licence

MPL-2.0. See [LICENSE.md](LICENSE.md).
