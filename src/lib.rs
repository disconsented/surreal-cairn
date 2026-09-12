//! # surreal-cairn
//!
//! > **Entirely AI-generated code.** Every line of this crate was written by
//! > Claude (Anthropic). No human wrote it and no human has audited it line by
//! > line. Read [the readme](https://github.com/disconsented/surreal-cairn)
//! > before you point it at a database you care about.
//!
//! A type-driven, async, up-only migration runner for [SurrealDB](https://surrealdb.com).
//!
//! ## Quickstart
//!
//! ```rust,no_run
//! use futures::StreamExt as _;
//! use surreal_cairn::{Migrator, Outcome};
//! use surrealdb::Surreal;
//! use surrealdb::engine::any::Any;
//!
//! # async fn example(db: &Surreal<Any>) -> Result<(), surreal_cairn::Error> {
//! let plan = Migrator::from_files("./migrations")?
//!     .with_table("_migrations")
//!     .ignore_checksum_changes(false)
//!     .validate()?
//!     .plan(db)
//!     .await?;
//!
//! println!("will apply {} migrations", plan.pending().len());
//!
//! // The stream is not Unpin; pin it before polling.
//! let mut stream = std::pin::pin!(plan.execute(db));
//! while let Some(outcome) = stream.next().await {
//!     match outcome? {
//!         Outcome::Applied { id, duration } => {
//!             println!("applied {id} in {:?}", duration);
//!         }
//!         Outcome::Skipped { id, .. } => {
//!             println!("skipped {id}");
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Typestate flow
//!
//! ```text
//! Migrator<Unvalidated>
//!     .with_table(...)        ─┐ builder methods only available here
//!     .ignore_checksum_changes(...)  ─┘
//!     .validate()?   →  Migrator<Validated>
//!     .plan(&db).await?  →  Plan
//!     .execute(&db)  →  impl Stream<Item = Result<Outcome, Error>>
//! ```
//!
//! - `validate()` is connection-free and checks for duplicate IDs and empty
//!   content.
//! - `plan()` queries the state table and classifies each migration.
//! - `execute()` drives the stream; each item is one migration outcome.
//!
//! ## Checksums
//!
//! Every migration's `content` is hashed with **BLAKE2b-256** (32-byte output,
//! hex-encoded) before it is stored in the state table. On the next run the
//! stored hash is compared against the hash of the file as it exists on disk. A
//! mismatch means the migration was edited after it was applied.
//!
//! Call `.ignore_checksum_changes(true)` to skip mismatched migrations with a
//! warning instead of returning [`Error::ChecksumMismatch`].

pub use error::Error;
pub use migrator::{Migrator, Unvalidated, Validated};
pub use plan::Plan;
pub use types::{Migration, Outcome, SkipReason};

mod checksum;
mod error;
mod migrator;
mod plan;
mod types;
