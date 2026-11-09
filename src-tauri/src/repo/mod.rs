//! SQLite access (SPEC §4.1). Every function that writes takes a [`Transaction`], so a logical
//! operation (including the FTS update) is atomic by construction; reads take a [`Connection`].
//!
//! Functions return `rusqlite::Result`: constraint violations surface as SQLite errors and are
//! mapped to IPC error codes by `error.rs`. "Not found" is `None` / `false`, never an error.

pub mod fts;
pub mod model;
pub mod reminders;
pub mod search;
pub mod settings;
pub mod tags;
pub mod tasks;

#[cfg(test)]
mod random_ops_tests;

use rusqlite::{Connection, Transaction, TransactionBehavior};

/// Runs `operation` in one `IMMEDIATE` transaction: committed on `Ok`, rolled back on `Err`.
/// Works with any error type that a SQLite error converts into (`rusqlite::Error`, `AppError`).
pub fn with_tx<T, E: From<rusqlite::Error>>(
    conn: &mut Connection,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, E>,
) -> Result<T, E> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let value = operation(&tx)?;
    tx.commit()?;
    Ok(value)
}
