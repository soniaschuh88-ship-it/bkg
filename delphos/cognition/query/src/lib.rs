//! # bkg-query — BQL: BKG Query Language engine
//! Single source of truth for all queries against DELPHOS world state.
//!
//! BQL: `SELECT tasks WHERE status = "blocked" AND depth > 3 ORDER BY entropy DESC LIMIT 10`
//!
//! The query engine operates against `bkg-state` RealmState entities.
//! It is intentionally simple and deterministic — no random query plans.

pub mod ast;
pub mod executor;
pub mod parser;
pub mod planner;
pub mod types;

pub use ast::{BqlExpr, BqlQuery, OrderDir};
pub use executor::{QueryExecutor, QueryResult};
pub use parser::{parse, ParseError};
pub use types::BqlValue;
