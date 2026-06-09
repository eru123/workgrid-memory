use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;
use workgrid_shared::errors::WorkGridError;

/// Global profile store (profiles.sqlite).
pub struct ProfileStore {
    db: Connection,
}

impl ProfileStore {
    pub fn open(db_path: &Path) -> Result<Self, WorkGridError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(db_path)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = ProfileStore { db };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), WorkGridError> {
        let schema = r#"
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                profile_type TEXT NOT NULL,
                description TEXT,
                sensitivity TEXT NOT NULL DEFAULT 'private',
                mcp_exposure TEXT NOT NULL DEFAULT 'disabled',
                source TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_reviewed_at TEXT,
                archived INTEGER DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS profile_attributes (
                id TEXT PRIMARY KEY, profile_id TEXT NOT NULL, key TEXT NOT NULL,
                value_json TEXT NOT NULL, sensitivity TEXT, source TEXT,
                confidence REAL DEFAULT 1.0, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS profile_instructions (
                id TEXT PRIMARY KEY, profile_id TEXT NOT NULL, name TEXT NOT NULL,
                trigger_terms_json TEXT, rules_json TEXT NOT NULL,
                examples_json TEXT, anti_patterns_json TEXT,
                priority INTEGER DEFAULT 100, enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS profile_relationships (
                id TEXT PRIMARY KEY, from_profile_id TEXT NOT NULL, to_profile_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL, confidence REAL DEFAULT 1.0,
                source TEXT, created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS profile_workspace_links (
                id TEXT PRIMARY KEY, profile_id TEXT NOT NULL, workspace_id TEXT NOT NULL,
                relevance TEXT, enabled INTEGER DEFAULT 1, created_at TEXT NOT NULL
            );
        "#;
        self.db.execute_batch(schema)?;
        Ok(())
    }

    pub fn create_profile(
        &self,
        name: &str,
        profile_type: &str,
        description: Option<&str>,
        sensitivity: &str,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.db.execute(
            "INSERT INTO profiles (id, name, profile_type, description, sensitivity, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?6)",
            params![&id, name, profile_type, description, sensitivity, &now],
        )?;
        Ok(id)
    }

    pub fn list_profiles(&self) -> Result<Vec<ProfileRow>, WorkGridError> {
        let mut stmt = self.db.prepare("SELECT id,name,profile_type,description,sensitivity,mcp_exposure,created_at,updated_at FROM profiles WHERE archived=0 ORDER BY updated_at DESC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProfileRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_type: row.get(2)?,
                    description: row.get(3)?,
                    sensitivity: row.get(4)?,
                    mcp_exposure: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_instruction(
        &self,
        profile_id: &str,
        name: &str,
        trigger_terms: &[&str],
        rules: &[&str],
        priority: i32,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let triggers = serde_json::to_string(trigger_terms).unwrap_or_default();
        let rules_json = serde_json::to_string(rules).unwrap_or_default();
        self.db.execute(
            "INSERT INTO profile_instructions (id,profile_id,name,trigger_terms_json,rules_json,priority,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?7)",
            params![&id, profile_id, name, &triggers, &rules_json, priority, &now],
        )?;
        Ok(id)
    }

    pub fn find_matching_instructions(
        &self,
        task: &str,
    ) -> Result<Vec<InstructionMatch>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT pi.id,pi.profile_id,pi.name,pi.trigger_terms_json,pi.rules_json,pi.priority,p.name FROM profile_instructions pi JOIN profiles p ON p.id=pi.profile_id WHERE pi.enabled=1 AND p.mcp_exposure='enabled' AND p.archived=0 ORDER BY pi.priority"
        )?;
        let all: Vec<InstructionMatch> = stmt
            .query_map([], |row| {
                Ok(InstructionMatch {
                    instruction_id: row.get(0)?,
                    profile_id: row.get(1)?,
                    name: row.get(2)?,
                    trigger_terms: row
                        .get::<_, String>(3)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    rules: row
                        .get::<_, String>(4)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    priority: row.get(5)?,
                    profile_name: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let task_lower = task.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|im| {
                im.trigger_terms
                    .iter()
                    .any(|t| task_lower.contains(&t.to_lowercase()))
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileRow {
    pub id: String,
    pub name: String,
    pub profile_type: String,
    pub description: Option<String>,
    pub sensitivity: String,
    pub mcp_exposure: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstructionMatch {
    pub instruction_id: String,
    pub profile_id: String,
    pub name: String,
    pub trigger_terms: Vec<String>,
    pub rules: Vec<String>,
    pub priority: i32,
    pub profile_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn temp_db() -> (ProfileStore, PathBuf) {
        let p = std::env::temp_dir().join(format!("p-{}.sqlite", Uuid::new_v4()));
        let store = ProfileStore::open(&p).unwrap();
        (store, p)
    }
    #[test]
    fn test_create_and_list() {
        let (store, p) = temp_db();
        store
            .create_profile("Test", "person", None, "private")
            .unwrap();
        assert_eq!(store.list_profiles().unwrap().len(), 1);
        std::fs::remove_file(&p).ok();
    }
    #[test]
    fn test_instruction_matching() {
        let (store, p) = temp_db();
        let pid = store
            .create_profile("Style", "skill", None, "enabled")
            .unwrap();
        // Set mcp_exposure after creation
        store
            .db
            .execute(
                "UPDATE profiles SET mcp_exposure='enabled', sensitivity='internal' WHERE id=?1",
                params![&pid],
            )
            .unwrap();
        store
            .add_instruction(&pid, "rules", &["commit"], &["imperative"], 10)
            .unwrap();
        let m = store.find_matching_instructions("make a commit").unwrap();
        assert_eq!(m.len(), 1);
        std::fs::remove_file(&p).ok();
    }
}
