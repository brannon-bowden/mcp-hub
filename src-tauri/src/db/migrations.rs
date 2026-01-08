//! Database Migration System
//!
//! Provides version-tracked schema migrations for safe database upgrades.
//! Each migration is stored in the migrations/ directory and applied in order.

use rusqlite::{Connection, Result as SqlResult};

/// Migration definitions - each entry is (version, name, sql)
/// Migrations are applied in order and tracked in the schema_version table
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "initial_schema",
        include_str!("migrations/001_initial_schema.sql"),
    ),
    (
        2,
        "add_indexes",
        include_str!("migrations/002_add_indexes.sql"),
    ),
];

/// Run all pending migrations on the database
///
/// This function:
/// 1. Creates the schema_version table if it doesn't exist
/// 2. Gets the current schema version
/// 3. Applies any migrations with version > current version
/// 4. Updates the schema version after each successful migration
pub fn run_migrations(conn: &Connection) -> SqlResult<()> {
    // Create schema_version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    // Get current schema version
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    log::info!("Current database schema version: {}", current_version);

    // Apply pending migrations
    for (version, name, sql) in MIGRATIONS.iter() {
        if *version > current_version {
            log::info!("Applying migration {} (v{})", name, version);

            // Execute the migration SQL
            conn.execute_batch(sql)?;

            // Record the migration
            conn.execute(
                "INSERT INTO schema_version (version, name, applied_at) VALUES (?1, ?2, datetime('now'))",
                rusqlite::params![version, name],
            )?;

            log::info!("Migration {} applied successfully", name);
        }
    }

    let final_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    log::info!("Database schema version after migrations: {}", final_version);

    Ok(())
}

/// Get the current schema version
#[allow(dead_code)]
pub fn get_schema_version(conn: &Connection) -> SqlResult<i64> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(0);
    }

    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
}

/// Check if there are pending migrations
#[allow(dead_code)]
pub fn has_pending_migrations(conn: &Connection) -> SqlResult<bool> {
    let current_version = get_schema_version(conn)?;
    let latest_version = MIGRATIONS.last().map(|(v, _, _)| *v).unwrap_or(0);
    Ok(current_version < latest_version)
}

/// Get list of applied migrations
#[allow(dead_code)]
pub fn get_applied_migrations(conn: &Connection) -> SqlResult<Vec<(i64, String, String)>> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare("SELECT version, name, applied_at FROM schema_version ORDER BY version")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_are_ordered() {
        let mut prev_version = 0;
        for (version, name, _) in MIGRATIONS.iter() {
            assert!(
                *version > prev_version,
                "Migration {} (v{}) must be greater than previous version {}",
                name,
                version,
                prev_version
            );
            prev_version = *version;
        }
    }

    #[test]
    fn test_run_migrations_on_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify schema_version table exists
        let version = get_schema_version(&conn).unwrap();
        assert!(version > 0);

        // Verify tables were created
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"servers".to_string()));
        assert!(tables.contains(&"client_instances".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        // Should still work and have correct version
        let version = get_schema_version(&conn).unwrap();
        assert!(version > 0);
    }

    #[test]
    fn test_has_pending_migrations() {
        let conn = Connection::open_in_memory().unwrap();

        // Before running migrations, there should be pending migrations
        assert!(has_pending_migrations(&conn).unwrap());

        // After running, no pending migrations
        run_migrations(&conn).unwrap();
        assert!(!has_pending_migrations(&conn).unwrap());
    }
}
