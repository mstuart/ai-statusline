use std::path::PathBuf;

use rusqlite::{params, Connection, Result as SqlResult};

/// A recorded session with aggregate cost data.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub model: String,
    pub total_cost: f64,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tokens_cached: u64,
}

/// A single cost event within a session.
#[derive(Debug, Clone)]
pub struct CostEvent {
    pub id: Option<i64>,
    pub session_id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub cost: f64,
    pub metadata: Option<String>,
}

/// Manages the local SQLite cost history database.
pub struct CostTracker {
    conn: Connection,
}

impl CostTracker {
    /// Open (or create) the history database at the default location.
    pub fn open() -> SqlResult<Self> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        let tracker = Self { conn };
        tracker.init_schema()?;
        Ok(tracker)
    }

    /// Open an in-memory database (for testing).
    #[cfg(test)]
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let tracker = Self { conn };
        tracker.init_schema()?;
        Ok(tracker)
    }

    fn db_path() -> PathBuf {
        dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("claude-status")
            .join("history.db")
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                model TEXT NOT NULL,
                total_cost REAL NOT NULL,
                tokens_input INTEGER NOT NULL,
                tokens_output INTEGER NOT NULL,
                tokens_cached INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                cost REAL NOT NULL,
                metadata TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_time ON sessions(start_time);
            CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);",
        )
    }

    /// Insert or update a session record.
    pub fn upsert_session(&self, session: &SessionRecord) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, start_time, end_time, model, total_cost, tokens_input, tokens_output, tokens_cached)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                end_time = excluded.end_time,
                model = excluded.model,
                total_cost = excluded.total_cost,
                tokens_input = excluded.tokens_input,
                tokens_output = excluded.tokens_output,
                tokens_cached = excluded.tokens_cached",
            params![
                session.id,
                session.start_time,
                session.end_time,
                session.model,
                session.total_cost,
                session.tokens_input as i64,
                session.tokens_output as i64,
                session.tokens_cached as i64,
            ],
        )?;
        Ok(())
    }

    /// Record a cost event.
    pub fn insert_event(&self, event: &CostEvent) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO events (session_id, timestamp, event_type, cost, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.session_id,
                event.timestamp,
                event.event_type,
                event.cost,
                event.metadata,
            ],
        )?;
        Ok(())
    }

    /// Get events since a given timestamp (Unix seconds).
    pub fn events_since(&self, since: i64) -> Vec<CostEvent> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, timestamp, event_type, cost, metadata
                 FROM events WHERE timestamp >= ?1 ORDER BY timestamp ASC",
            )
            .unwrap();

        stmt.query_map(params![since], |row| {
            Ok(CostEvent {
                id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp: row.get(2)?,
                event_type: row.get(3)?,
                cost: row.get(4)?,
                metadata: row.get(5)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Total cost of events since a given timestamp.
    pub fn total_cost_since(&self, since: i64) -> f64 {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(cost), 0.0) FROM events WHERE timestamp >= ?1",
                params![since],
                |row| row.get(0),
            )
            .unwrap_or(0.0)
    }

    /// Total cost from sessions in a time range.
    pub fn session_cost_range(&self, from: i64, to: i64) -> f64 {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(total_cost), 0.0) FROM sessions
                 WHERE start_time >= ?1 AND start_time < ?2",
                params![from, to],
                |row| row.get(0),
            )
            .unwrap_or(0.0)
    }

    /// Get sessions in a time range ordered by cost (descending).
    pub fn top_sessions(&self, from: i64, to: i64, limit: u32) -> Vec<SessionRecord> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, start_time, end_time, model, total_cost, tokens_input, tokens_output, tokens_cached
                 FROM sessions WHERE start_time >= ?1 AND start_time < ?2
                 ORDER BY total_cost DESC LIMIT ?3",
            )
            .unwrap();

        stmt.query_map(params![from, to, limit], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                model: row.get(3)?,
                total_cost: row.get(4)?,
                tokens_input: row.get::<_, i64>(5)? as u64,
                tokens_output: row.get::<_, i64>(6)? as u64,
                tokens_cached: row.get::<_, i64>(7)? as u64,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Count of sessions in a time range.
    pub fn session_count_range(&self, from: i64, to: i64) -> u64 {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE start_time >= ?1 AND start_time < ?2",
                params![from, to],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) as u64
    }

    /// Get the current session by session_id.
    pub fn get_session(&self, session_id: &str) -> Option<SessionRecord> {
        self.conn
            .query_row(
                "SELECT id, start_time, end_time, model, total_cost, tokens_input, tokens_output, tokens_cached
                 FROM sessions WHERE id = ?1",
                params![session_id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        start_time: row.get(1)?,
                        end_time: row.get(2)?,
                        model: row.get(3)?,
                        total_cost: row.get(4)?,
                        tokens_input: row.get::<_, i64>(5)? as u64,
                        tokens_output: row.get::<_, i64>(6)? as u64,
                        tokens_cached: row.get::<_, i64>(7)? as u64,
                    })
                },
            )
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_query_session() {
        let tracker = CostTracker::open_in_memory().unwrap();

        let session = SessionRecord {
            id: "test-session-1".into(),
            start_time: 1000,
            end_time: Some(2000),
            model: "claude-sonnet-4-5-20250929".into(),
            total_cost: 0.45,
            tokens_input: 5000,
            tokens_output: 1200,
            tokens_cached: 3000,
        };

        tracker.upsert_session(&session).unwrap();

        let fetched = tracker.get_session("test-session-1").unwrap();
        assert_eq!(fetched.total_cost, 0.45);
        assert_eq!(fetched.tokens_input, 5000);
    }

    #[test]
    fn test_insert_events_and_query() {
        let tracker = CostTracker::open_in_memory().unwrap();

        let session = SessionRecord {
            id: "s1".into(),
            start_time: 100,
            end_time: None,
            model: "claude-opus-4-6".into(),
            total_cost: 1.0,
            tokens_input: 10000,
            tokens_output: 2000,
            tokens_cached: 5000,
        };
        tracker.upsert_session(&session).unwrap();

        for i in 0..5 {
            tracker
                .insert_event(&CostEvent {
                    id: None,
                    session_id: "s1".into(),
                    timestamp: 100 + i * 10,
                    event_type: "message".into(),
                    cost: 0.10,
                    metadata: None,
                })
                .unwrap();
        }

        let events = tracker.events_since(120);
        assert_eq!(events.len(), 3);

        let total = tracker.total_cost_since(100);
        assert!((total - 0.50).abs() < 0.001);
    }

    #[test]
    fn test_top_sessions() {
        let tracker = CostTracker::open_in_memory().unwrap();

        for i in 0..5 {
            tracker
                .upsert_session(&SessionRecord {
                    id: format!("s{}", i),
                    start_time: 1000 + i * 100,
                    end_time: None,
                    model: "claude-sonnet-4-5-20250929".into(),
                    total_cost: (i as f64) * 5.0,
                    tokens_input: 1000,
                    tokens_output: 200,
                    tokens_cached: 500,
                })
                .unwrap();
        }

        let top = tracker.top_sessions(0, 2000, 3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].id, "s4"); // highest cost
        assert_eq!(top[1].id, "s3");
        assert_eq!(top[2].id, "s2");
    }

    #[test]
    fn test_session_cost_range() {
        let tracker = CostTracker::open_in_memory().unwrap();

        tracker
            .upsert_session(&SessionRecord {
                id: "a".into(),
                start_time: 500,
                end_time: None,
                model: "opus".into(),
                total_cost: 10.0,
                tokens_input: 0,
                tokens_output: 0,
                tokens_cached: 0,
            })
            .unwrap();
        tracker
            .upsert_session(&SessionRecord {
                id: "b".into(),
                start_time: 1500,
                end_time: None,
                model: "sonnet".into(),
                total_cost: 5.0,
                tokens_input: 0,
                tokens_output: 0,
                tokens_cached: 0,
            })
            .unwrap();

        let cost = tracker.session_cost_range(0, 1000);
        assert!((cost - 10.0).abs() < 0.001);

        let cost = tracker.session_cost_range(0, 2000);
        assert!((cost - 15.0).abs() < 0.001);
    }
}
