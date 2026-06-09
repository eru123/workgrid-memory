use rusqlite::{params, Connection};
use std::path::Path;
use std::process::Command;
use workgrid_shared::types::{Workspace, WorkspaceStatus};
use workgrid_shared::errors::WorkGridError;

/// Manages the global app database (app.sqlite) and the workspace registry.
pub struct MetadataStore {
    db: Connection,
}

impl MetadataStore {
    /// Open or create the app database at the given path.
    pub fn open(db_path: &Path) -> Result<Self, WorkGridError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Connection::open(db_path)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let store = MetadataStore { db };
        store.initialize_schema()?;
        Ok(store)
    }

    /// Create the global app schema if it doesn't exist.
    fn initialize_schema(&self) -> Result<(), WorkGridError> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                git_remote TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_indexed_at TEXT,
                status TEXT NOT NULL DEFAULT 'new'
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    /// Add a new workspace. Returns the created workspace.
    pub fn add_workspace(&self, name: &str, root_path: &str, git_remote: Option<&str>) -> Result<Workspace, WorkGridError> {
        // Canonicalize and validate path
        let canonical = std::fs::canonicalize(root_path)
            .map_err(|_| WorkGridError::WorkspacePathNotFound(root_path.to_string()))?;
        let canonical_str = canonical.to_string_lossy().to_string();

        if !canonical.is_dir() {
            return Err(WorkGridError::WorkspacePathNotFound(root_path.to_string()));
        }

        // Check for duplicate
        let exists: bool = self.db.query_row(
            "SELECT COUNT(*) > 0 FROM workspaces WHERE root_path = ?1",
            params![&canonical_str],
            |row| row.get(0),
        )?;

        if exists {
            return Err(WorkGridError::DuplicateWorkspace(canonical_str));
        }

        let id = Self::generate_workspace_id(&canonical_str, git_remote);
        let now = chrono::Utc::now().to_rfc3339();

        self.db.execute(
            "INSERT INTO workspaces (id, name, root_path, git_remote, created_at, updated_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, 'new')",
            params![&id, name, &canonical_str, git_remote, &now],
        )?;

        Ok(self.get_workspace(&id)?.unwrap())
    }

    /// List all registered workspaces.
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, root_path, git_remote, created_at, updated_at, last_indexed_at, status
             FROM workspaces
             ORDER BY created_at DESC"
        )?;

        let workspaces = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                git_remote: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                last_indexed_at: row.get(6)?,
                status: {
                    let s: String = row.get(7)?;
                    Self::parse_status(&s)
                },
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(workspaces)
    }

    /// Get a specific workspace by ID.
    pub fn get_workspace(&self, id: &str) -> Result<Option<Workspace>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, root_path, git_remote, created_at, updated_at, last_indexed_at, status
             FROM workspaces WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                git_remote: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                last_indexed_at: row.get(6)?,
                status: {
                    let s: String = row.get(7)?;
                    Self::parse_status(&s)
                },
            })
        })?;

        match rows.next() {
            Some(Ok(ws)) => Ok(Some(ws)),
            Some(Err(e)) => Err(WorkGridError::Database(e.to_string())),
            None => Ok(None),
        }
    }

    /// Remove a workspace by ID. Does NOT delete workspace data on disk.
    pub fn remove_workspace(&self, id: &str) -> Result<(), WorkGridError> {
        let affected = self.db.execute(
            "DELETE FROM workspaces WHERE id = ?1",
            params![id],
        )?;

        if affected == 0 {
            return Err(WorkGridError::WorkspaceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Update workspace status.
    pub fn update_workspace_status(&self, id: &str, status: WorkspaceStatus) -> Result<(), WorkGridError> {
        let status_str = Self::status_to_str(&status);
        let now = chrono::Utc::now().to_rfc3339();

        let affected = self.db.execute(
            "UPDATE workspaces SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status_str, &now, id],
        )?;

        if affected == 0 {
            return Err(WorkGridError::WorkspaceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Update the last_indexed_at timestamp.
    pub fn touch_indexed(&self, id: &str) -> Result<(), WorkGridError> {
        let now = chrono::Utc::now().to_rfc3339();

        let affected = self.db.execute(
            "UPDATE workspaces SET last_indexed_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![&now, id],
        )?;

        if affected == 0 {
            return Err(WorkGridError::WorkspaceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Generate a stable workspace ID from path + optional git remote.
    fn generate_workspace_id(root_path: &str, git_remote: Option<&str>) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        root_path.hash(&mut hasher);
        if let Some(remote) = git_remote {
            remote.hash(&mut hasher);
        }
        let hash = hasher.finish();
        format!("ws-{:016x}", hash)
    }

    fn parse_status(s: &str) -> WorkspaceStatus {
        match s {
            "indexing" => WorkspaceStatus::Indexing,
            "ready" => WorkspaceStatus::Ready,
            "degraded" => WorkspaceStatus::Degraded,
            "stale" => WorkspaceStatus::Stale,
            "error" => WorkspaceStatus::Error,
            "paused" => WorkspaceStatus::Paused,
            _ => WorkspaceStatus::New,
        }
    }

    /// Attempt to detect the git remote URL for a given workspace path.
    pub fn detect_git_remote(root_path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["-C", root_path, "remote", "get-url", "origin"])
            .output()
            .ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    fn status_to_str(status: &WorkspaceStatus) -> &str {
        match status {
            WorkspaceStatus::New => "new",
            WorkspaceStatus::Indexing => "indexing",
            WorkspaceStatus::Ready => "ready",
            WorkspaceStatus::Degraded => "degraded",
            WorkspaceStatus::Stale => "stale",
            WorkspaceStatus::Error => "error",
            WorkspaceStatus::Paused => "paused",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    fn temp_db_path() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("workgrid-test");
        std::fs::create_dir_all(&dir).unwrap();
        let name = format!("test-{}.sqlite", Uuid::new_v4());
        dir.join(name)
    }

    #[test]
    fn test_add_and_list_workspaces() {
        let db_path = temp_db_path();
        let store = MetadataStore::open(&db_path).unwrap();
        let tmp_dir = std::env::temp_dir();
        let ws = store
            .add_workspace("Test Project", tmp_dir.to_str().unwrap(), None)
            .unwrap();
        assert_eq!(ws.name, "Test Project");
        let list = store.list_workspaces().unwrap();
        assert_eq!(list.len(), 1);
        let fetched = store.get_workspace(&ws.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test Project");
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_duplicate_workspace() {
        let db_path = temp_db_path();
        let store = MetadataStore::open(&db_path).unwrap();
        let tmp_dir = std::env::temp_dir();
        store.add_workspace("First", tmp_dir.to_str().unwrap(), None).unwrap();
        let result = store.add_workspace("Second", tmp_dir.to_str().unwrap(), None);
        assert!(result.is_err());
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_remove_workspace() {
        let db_path = temp_db_path();
        let store = MetadataStore::open(&db_path).unwrap();
        let tmp_dir = std::env::temp_dir();
        let ws = store.add_workspace("To Remove", tmp_dir.to_str().unwrap(), None).unwrap();
        store.remove_workspace(&ws.id).unwrap();
        assert!(store.list_workspaces().unwrap().is_empty());
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_bad_path() {
        let db_path = temp_db_path();
        let store = MetadataStore::open(&db_path).unwrap();
        let result = store.add_workspace("Nope", "/nonexistent/xyz", None);
        assert!(result.is_err());
        std::fs::remove_file(&db_path).ok();
    }
}
