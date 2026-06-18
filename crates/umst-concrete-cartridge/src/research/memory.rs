// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Research memory store — functional append, pure query filter.

use super::geometry::{mix_l1_distance, morton_index_distance};
use super::types::{MemoryQuery, MemoryRecord};
use std::path::Path;
use thiserror::Error;

/// Memory store append / lookup failures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Store boundary error sum type; admissibility on accept path.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("duplicate content_id: {0}")]
    DuplicateContentId(String),
    #[error("duplicate idempotency_key: {0}")]
    DuplicateIdempotencyKey(String),
    #[error("memory_id not found: {0}")]
    NotFound(String),
    #[error("sqlite: {0}")]
    Sqlite(String),
}

/// Alias for [`StoreError`] on promotion/MCP paths.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Type alias for ergonomic imports; same variants as `StoreError`.
pub type MemoryError = StoreError;

/// Pure lookup by `memory_id` over current store rows.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Linear scan morphism; no interior mutation.
#[must_use]
pub fn find_by_memory_id(
    store: &ResearchStore,
    memory_id: &str,
) -> Result<MemoryRecord, StoreError> {
    store
        .rows()
        .into_iter()
        .find(|r| r.memory_id.as_deref() == Some(memory_id))
        .ok_or_else(|| StoreError::NotFound(memory_id.to_string()))
}

/// Functional memory store port — append returns new store value.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Type-class for functional append; no global mutable store.
pub trait MemoryStore {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Append one row; returns new store (functional update).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Functional append morphism; duplicate content_id rejected.
    fn append(self, record: MemoryRecord) -> Result<(Self, ()), Self::Error>
    where
        Self: Sized;

    /// Snapshot current rows (pure read).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Immutable row snapshot for pure `filter_records`.
    fn rows(&self) -> Vec<MemoryRecord>;
}

/// Pure filter morphism over memory rows.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Query filter over row slice; L1/Morton sort when requested.
#[must_use]
pub fn filter_records(rows: &[MemoryRecord], query: &MemoryQuery) -> Vec<MemoryRecord> {
    let mut out: Vec<MemoryRecord> = rows
        .iter()
        .filter(|r| {
            if query.admissible_only && !r.payload.gate_summary.admissible {
                return false;
            }
            if let Some(regime) = &query.curing_regime {
                let actual = r
                    .payload
                    .process
                    .get("curing_regime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if actual != regime {
                    return false;
                }
            }
            if let Some(idx) = query.hilbert_index {
                let max_d = query.max_hilbert_distance.unwrap_or(0);
                let Some(geom) = r.mix_geometry.as_ref() else {
                    return false;
                };
                if morton_index_distance(geom.hilbert_index, idx) > max_d {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    if let Some(ref near) = query.near_mix_spec {
        if let Some(max_l1) = query.max_mix_l1 {
            out.retain(|r| {
                mix_l1_distance(near, &r.payload.mix_spec)
                    .map(|d| d <= max_l1)
                    .unwrap_or(false)
            });
        }
        out.sort_by(|a, b| {
            let da = mix_l1_distance(near, &a.payload.mix_spec).unwrap_or(f64::MAX);
            let db = mix_l1_distance(near, &b.payload.mix_spec).unwrap_or(f64::MAX);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    out.into_iter()
        .take(query.limit.unwrap_or(usize::MAX))
        .collect()
}

/// In-memory store — append returns a new store value (no interior mutation).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Functional in-memory store; rows + idempotency keys owned by value.
#[derive(Debug, Clone, Default)]
pub struct InMemoryStore {
    rows: Vec<MemoryRecord>,
    idempotency_keys: Vec<String>,
}

impl InMemoryStore {
    /// Empty in-memory store.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Zero-row initializer for tests and default session.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: Vec::new(),
            idempotency_keys: Vec::new(),
        }
    }

    /// Construct store from existing rows (tests / hydration).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Test/fixture constructor; no duplicate checks on load.
    #[must_use]
    pub fn from_rows(rows: Vec<MemoryRecord>) -> Self {
        Self {
            rows,
            idempotency_keys: Vec::new(),
        }
    }

    /// Pure query over current rows.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Delegates to `filter_records` on owned row slice.
    #[must_use]
    pub fn query(&self, q: &MemoryQuery) -> Vec<MemoryRecord> {
        filter_records(&self.rows, q)
    }
}

impl MemoryStore for InMemoryStore {
    type Error = StoreError;

    fn append(mut self, record: MemoryRecord) -> Result<(Self, ()), Self::Error> {
        if self.rows.iter().any(|r| r.content_id == record.content_id) {
            return Err(StoreError::DuplicateContentId(record.content_id));
        }
        self.rows.push(record);
        Ok((self, ()))
    }

    fn rows(&self) -> Vec<MemoryRecord> {
        self.rows.clone()
    }
}

/// Unified research store — in-memory or SQLite (`UMST_MEMORY_DB`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Store enum dispatch; SQLite arm is IO boundary.
#[derive(Debug, Clone)]
pub enum ResearchStore {
    InMemory(InMemoryStore),
    Sqlite(SqliteStore),
}

impl Default for ResearchStore {
    fn default() -> Self {
        Self::InMemory(InMemoryStore::new())
    }
}

impl ResearchStore {
    /// Default in-memory research store.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Constructs functional in-memory arm only.
    #[must_use]
    pub fn in_memory() -> Self {
        Self::InMemory(InMemoryStore::new())
    }

    /// Open SQLite-backed store at path (IO boundary).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Filesystem + SQLite connection open; not pure morphism.
    pub fn open_sqlite(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Ok(Self::Sqlite(SqliteStore::open(path.as_ref())?))
    }

    /// Resolve store from `UMST_MEMORY_DB` env or default in-memory.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Environment-driven store selection at session start.
    pub fn from_env() -> Result<Self, StoreError> {
        match std::env::var("UMST_MEMORY_DB") {
            Ok(path) if !path.is_empty() => Self::open_sqlite(path),
            _ => Ok(Self::default()),
        }
    }

    /// Query rows through active store backend.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Dispatches to in-memory or SQLite IO arm.
    #[must_use]
    pub fn query(&self, q: &MemoryQuery) -> Vec<MemoryRecord> {
        match self {
            Self::InMemory(s) => s.query(q),
            Self::Sqlite(s) => s.query(q).unwrap_or_default(),
        }
    }

    /// Functional append with optional idempotency key.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Store dispatch + idempotency check at IO boundary.
    pub fn append(
        self,
        record: MemoryRecord,
        idempotency_key: Option<&str>,
    ) -> Result<(Self, ()), StoreError> {
        if let Some(key) = idempotency_key {
            if self.has_idempotency_key(key) {
                return Err(StoreError::DuplicateIdempotencyKey(key.to_string()));
            }
        }
        match self {
            Self::InMemory(mut s) => {
                if let Some(key) = idempotency_key {
                    s.idempotency_keys.push(key.to_string());
                }
                let (s, ()) = s.append(record)?;
                Ok((Self::InMemory(s), ()))
            }
            Self::Sqlite(s) => {
                let (s, ()) = s.append(record, idempotency_key)?;
                Ok((Self::Sqlite(s), ()))
            }
        }
    }

    /// Snapshot all rows from active backend.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: May load from SQLite; pure filter on result in `filter_records`.
    #[must_use]
    pub fn rows(&self) -> Vec<MemoryRecord> {
        match self {
            Self::InMemory(s) => s.rows(),
            Self::Sqlite(s) => s.rows().unwrap_or_default(),
        }
    }

    fn has_idempotency_key(&self, key: &str) -> bool {
        match self {
            Self::InMemory(s) => s.idempotency_keys.iter().any(|k| k == key),
            Self::Sqlite(s) => s.has_idempotency_key(key).unwrap_or(false),
        }
    }
}

#[cfg(feature = "agent-layer")]
mod sqlite_store {
    use super::super::types::{MemoryQuery, MemoryRecord};
    use super::{filter_records, MemoryStore, StoreError};
    use rusqlite::{params, Connection};
    use serde_json;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    const SCHEMA_SQL: &str = "
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS memory_records (
          memory_id TEXT PRIMARY KEY,
          content_id TEXT UNIQUE NOT NULL,
          idempotency_key TEXT UNIQUE,
          record_json TEXT NOT NULL
        ) STRICT;
        CREATE TRIGGER IF NOT EXISTS memory_records_no_update
          BEFORE UPDATE ON memory_records
        BEGIN
          SELECT RAISE(ABORT, 'memory_records are immutable');
        END;
        CREATE TRIGGER IF NOT EXISTS memory_records_no_delete
          BEFORE DELETE ON memory_records
        BEGIN
          SELECT RAISE(ABORT, 'memory_records are immutable');
        END;
    ";

    /// SQLite-backed store — connection mutation isolated here (effect boundary).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Durable store IO; query uses pure `filter_records` on load.
    #[derive(Debug, Clone)]
    pub struct SqliteStore {
        conn: Arc<Connection>,
        path: PathBuf,
    }

    impl SqliteStore {
        /// Open or create SQLite memory database at path.
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: Connection + schema migration IO boundary.
        pub fn open(path: &Path) -> Result<Self, StoreError> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| StoreError::Sqlite(e.to_string()))?;
            }
            let conn = Connection::open(path).map_err(|e| StoreError::Sqlite(e.to_string()))?;
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            Ok(Self {
                conn: Arc::new(conn),
                path: path.to_path_buf(),
            })
        }

        /// Database file path.
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: Path accessor for operator diagnostics only.
        pub fn path(&self) -> &Path {
            &self.path
        }

        fn load_rows(&self) -> Result<Vec<MemoryRecord>, StoreError> {
            let mut stmt = self
                .conn
                .prepare("SELECT record_json FROM memory_records ORDER BY rowid")
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    let text: String = row.get(0)?;
                    Ok(text)
                })
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            rows.map(|r| {
                let text = r.map_err(|e| StoreError::Sqlite(e.to_string()))?;
                serde_json::from_str(&text).map_err(|e| StoreError::Sqlite(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
        }

        /// Check whether idempotency key already exists.
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: SQLite read for duplicate suppression at append.
        pub fn has_idempotency_key(&self, key: &str) -> Result<bool, StoreError> {
            let mut stmt = self
                .conn
                .prepare("SELECT 1 FROM memory_records WHERE idempotency_key = ?1 LIMIT 1")
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            let found = stmt
                .exists(params![key])
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            Ok(found)
        }

        /// Append one immutable memory row (IO).
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: SQLite INSERT; rows must pass accept gate before call.
        pub fn append(
            self,
            record: MemoryRecord,
            idempotency_key: Option<&str>,
        ) -> Result<(Self, ()), StoreError> {
            let memory_id = record
                .memory_id
                .clone()
                .ok_or_else(|| StoreError::Sqlite("missing memory_id".into()))?;
            let json =
                serde_json::to_string(&record).map_err(|e| StoreError::Sqlite(e.to_string()))?;
            self.conn
                .execute(
                    "INSERT INTO memory_records (memory_id, content_id, idempotency_key, record_json) VALUES (?1, ?2, ?3, ?4)",
                    params![memory_id, record.content_id, idempotency_key, json],
                )
                .map_err(|e| StoreError::Sqlite(e.to_string()))?;
            Ok((self, ()))
        }

        /// Load rows and apply pure `filter_records`.
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: SQLite SELECT + pure filter morphism.
        pub fn query(&self, q: &MemoryQuery) -> Result<Vec<MemoryRecord>, StoreError> {
            let rows = self.load_rows()?;
            Ok(filter_records(&rows, q))
        }

        /// Load all memory rows from database.
        /// formal_anchor: NONE
        /// formal_status: NONE
        /// formal_anchor_rationale: Full table scan IO; filter in caller if needed.
        pub fn rows(&self) -> Result<Vec<MemoryRecord>, StoreError> {
            self.load_rows()
        }
    }

    impl MemoryStore for SqliteStore {
        type Error = StoreError;

        fn append(self, record: MemoryRecord) -> Result<(Self, ()), Self::Error> {
            self.append(record, None)
        }

        fn rows(&self) -> Vec<MemoryRecord> {
            self.load_rows().unwrap_or_default()
        }
    }
}

#[cfg(feature = "agent-layer")]
/// Re-export SQLite store when `agent-layer` feature is enabled.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export of IO-backed store; see `sqlite_store::SqliteStore`.
pub use sqlite_store::SqliteStore;
