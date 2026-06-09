//! Global profile store — manages profiles, attributes, instructions,
//! assets, relationships, workspace links, and audit logging.

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
            CREATE TABLE IF NOT EXISTS profile_assets (
                id TEXT PRIMARY KEY, profile_id TEXT NOT NULL,
                asset_type TEXT NOT NULL, local_path TEXT NOT NULL,
                hash TEXT, description TEXT,
                sensitivity TEXT NOT NULL DEFAULT 'sensitive', created_at TEXT NOT NULL
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
            CREATE TABLE IF NOT EXISTS profile_audit_log (
                id TEXT PRIMARY KEY, profile_id TEXT, action TEXT NOT NULL,
                actor TEXT NOT NULL, detail_json TEXT, created_at TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS profiles_fts USING fts5(
                name, profile_type, description, searchable_text
            );
        "#;
        self.db.execute_batch(schema)?;
        Ok(())
    }

    fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    // ── Profile CRUD ──

    pub fn create_profile(
        &self,
        name: &str,
        profile_type: &str,
        description: Option<&str>,
        sensitivity: &str,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
        self.db.execute(
            "INSERT INTO profiles (id, name, profile_type, description, sensitivity, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?6)",
            params![&id, name, profile_type, description, sensitivity, &now],
        )?;
        // Update FTS
        self.db.execute(
            "INSERT INTO profiles_fts (name, profile_type, description, searchable_text)
             VALUES (?1,?2,?3,?4)",
            params![name, profile_type, description.unwrap_or(""), ""],
        )?;
        self.audit(
            &id,
            "create",
            "system",
            &format!("Created profile {} ({})", name, profile_type),
        )?;
        Ok(id)
    }

    pub fn update_profile(
        &self,
        id: &str,
        name: &str,
        profile_type: &str,
        description: Option<&str>,
        sensitivity: &str,
    ) -> Result<(), WorkGridError> {
        let now = Self::now();
        self.db.execute(
            "UPDATE profiles SET name=?1, profile_type=?2, description=?3, sensitivity=?4, updated_at=?5 WHERE id=?6",
            params![name, profile_type, description, sensitivity, &now, id],
        )?;
        self.audit(id, "update", "system", "Updated profile")?;
        Ok(())
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), WorkGridError> {
        self.db
            .execute("DELETE FROM profiles WHERE id=?1", params![id])?;
        self.db.execute(
            "DELETE FROM profile_attributes WHERE profile_id=?1",
            params![id],
        )?;
        self.db.execute(
            "DELETE FROM profile_instructions WHERE profile_id=?1",
            params![id],
        )?;
        self.db.execute(
            "DELETE FROM profile_assets WHERE profile_id=?1",
            params![id],
        )?;
        self.db.execute(
            "DELETE FROM profile_relationships WHERE from_profile_id=?1 OR to_profile_id=?2",
            params![id, id],
        )?;
        self.db.execute(
            "DELETE FROM profile_workspace_links WHERE profile_id=?1",
            params![id],
        )?;
        self.audit(
            id,
            "delete",
            "system",
            "Deleted profile and all related data",
        )?;
        Ok(())
    }

    pub fn archive_profile(&self, id: &str) -> Result<(), WorkGridError> {
        self.db.execute(
            "UPDATE profiles SET archived=1, updated_at=?1 WHERE id=?2",
            params![Self::now(), id],
        )?;
        self.audit(id, "archive", "system", "Archived profile")?;
        Ok(())
    }

    pub fn get_profile(&self, id: &str) -> Result<Option<ProfileRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id,name,profile_type,description,sensitivity,mcp_exposure,source,created_at,updated_at,last_reviewed_at,archived FROM profiles WHERE id=?1",
        )?;
        let row = stmt
            .query_map(params![id], |row| {
                Ok(ProfileRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_type: row.get(2)?,
                    description: row.get(3)?,
                    sensitivity: row.get(4)?,
                    mcp_exposure: row.get(5)?,
                    source: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_reviewed_at: row.get(9)?,
                    archived: row.get::<_, i32>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(row.into_iter().next())
    }

    pub fn list_profiles(&self) -> Result<Vec<ProfileRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id,name,profile_type,description,sensitivity,mcp_exposure,source,created_at,updated_at,last_reviewed_at,archived FROM profiles WHERE archived=0 ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProfileRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_type: row.get(2)?,
                    description: row.get(3)?,
                    sensitivity: row.get(4)?,
                    mcp_exposure: row.get(5)?,
                    source: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_reviewed_at: row.get(9)?,
                    archived: row.get::<_, i32>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn set_mcp_exposure(&self, profile_id: &str, exposure: &str) -> Result<(), WorkGridError> {
        self.db.execute(
            "UPDATE profiles SET mcp_exposure=?1, updated_at=?2 WHERE id=?3",
            params![exposure, Self::now(), profile_id],
        )?;
        self.audit(
            profile_id,
            "mcp_exposure",
            "system",
            &format!("MCP exposure set to {}", exposure),
        )?;
        Ok(())
    }

    // ── Profile search ──

    pub fn search_profiles_fts(&self, query: &str) -> Result<Vec<ProfileRow>, WorkGridError> {
        let safe = query
            .chars()
            .filter(|&c| c.is_alphanumeric() || c == '_' || c == ' ')
            .collect::<String>();
        let fts_query = format!("{}*", safe);
        let sql = format!(
            "SELECT p.id,p.name,p.profile_type,p.description,p.sensitivity,p.mcp_exposure,p.source,p.created_at,p.updated_at,p.last_reviewed_at,p.archived
             FROM profiles_fts fts JOIN profiles p ON p.name = fts.name
             WHERE profiles_fts MATCH ?1 AND p.archived=0
             ORDER BY rank LIMIT 20"
        );
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt
            .query_map(params![&fts_query], |row| {
                Ok(ProfileRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_type: row.get(2)?,
                    description: row.get(3)?,
                    sensitivity: row.get(4)?,
                    mcp_exposure: row.get(5)?,
                    source: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_reviewed_at: row.get(9)?,
                    archived: row.get::<_, i32>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Attributes ──

    pub fn set_attribute(
        &self,
        profile_id: &str,
        key: &str,
        value: &serde_json::Value,
        sensitivity: Option<&str>,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
        let value_str = serde_json::to_string(value).unwrap_or_default();
        self.db.execute(
            "INSERT OR REPLACE INTO profile_attributes (id, profile_id, key, value_json, sensitivity, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?6)",
            params![&id, profile_id, key, &value_str, sensitivity, &now],
        )?;
        Ok(id)
    }

    pub fn get_attributes(&self, profile_id: &str) -> Result<Vec<AttributeRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id, key, value_json, sensitivity FROM profile_attributes WHERE profile_id=?1",
        )?;
        let rows = stmt
            .query_map(params![profile_id], |row| {
                Ok(AttributeRow {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    value: row
                        .get::<_, String>(2)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    sensitivity: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Instructions ──

    pub fn add_instruction(
        &self,
        profile_id: &str,
        name: &str,
        trigger_terms: &[&str],
        rules: &[&str],
        priority: i32,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
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
            "SELECT pi.id,pi.profile_id,pi.name,pi.trigger_terms_json,pi.rules_json,pi.priority,p.name FROM profile_instructions pi JOIN profiles p ON p.id=pi.profile_id WHERE pi.enabled=1 AND p.mcp_exposure='enabled' AND p.archived=0 ORDER BY pi.priority",
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

    // ── Relationships ──

    pub fn add_relationship(
        &self,
        from_profile_id: &str,
        to_profile_id: &str,
        relationship_type: &str,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
        self.db.execute(
            "INSERT INTO profile_relationships (id,from_profile_id,to_profile_id,relationship_type,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![&id, from_profile_id, to_profile_id, relationship_type, &now],
        )?;
        Ok(id)
    }

    pub fn get_relationships(
        &self,
        profile_id: &str,
    ) -> Result<Vec<RelationshipRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT r.id, r.from_profile_id, r.to_profile_id, r.relationship_type, r.confidence,
                    p.name as other_name
             FROM profile_relationships r
             JOIN profiles p ON (
                 (p.id = r.to_profile_id AND r.from_profile_id = ?1) OR
                 (p.id = r.from_profile_id AND r.to_profile_id = ?1)
             )
             WHERE p.id != ?1",
        )?;
        let rows = stmt
            .query_map(params![profile_id], |row| {
                Ok(RelationshipRow {
                    id: row.get(0)?,
                    from_id: row.get(1)?,
                    to_id: row.get(2)?,
                    relationship_type: row.get(3)?,
                    confidence: row.get(4)?,
                    other_name: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Workspace links ──

    pub fn link_workspace(
        &self,
        profile_id: &str,
        workspace_id: &str,
        relevance: Option<&str>,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
        self.db.execute(
            "INSERT INTO profile_workspace_links (id,profile_id,workspace_id,relevance,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![&id, profile_id, workspace_id, relevance, &now],
        )?;
        Ok(id)
    }

    pub fn get_workspace_links(
        &self,
        profile_id: &str,
    ) -> Result<Vec<WorkspaceLinkRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT id, workspace_id, relevance, enabled FROM profile_workspace_links WHERE profile_id=?1",
        )?;
        let rows = stmt
            .query_map(params![profile_id], |row| {
                Ok(WorkspaceLinkRow {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    relevance: row.get(2)?,
                    enabled: row.get::<_, i32>(3)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_profiles_for_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<ProfileRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT p.id,p.name,p.profile_type,p.description,p.sensitivity,p.mcp_exposure,p.source,p.created_at,p.updated_at,p.last_reviewed_at,p.archived
             FROM profiles p
             JOIN profile_workspace_links l ON l.profile_id = p.id
             WHERE l.workspace_id = ?1 AND l.enabled = 1 AND p.archived = 0 AND p.mcp_exposure = 'enabled'
             ORDER BY p.updated_at DESC",
        )?;
        let rows = stmt
            .query_map(params![workspace_id], |row| {
                Ok(ProfileRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_type: row.get(2)?,
                    description: row.get(3)?,
                    sensitivity: row.get(4)?,
                    mcp_exposure: row.get(5)?,
                    source: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_reviewed_at: row.get(9)?,
                    archived: row.get::<_, i32>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Audit ──

    fn audit(
        &self,
        profile_id: &str,
        action: &str,
        actor: &str,
        detail: &str,
    ) -> Result<(), WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now();
        self.db.execute(
            "INSERT INTO profile_audit_log (id,profile_id,action,actor,detail_json,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![&id, profile_id, action, actor, detail, &now],
        )?;
        Ok(())
    }
}

// ── Row types ──

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileRow {
    pub id: String,
    pub name: String,
    pub profile_type: String,
    pub description: Option<String>,
    pub sensitivity: String,
    pub mcp_exposure: String,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_reviewed_at: Option<String>,
    pub archived: bool,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct AttributeRow {
    pub id: String,
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub sensitivity: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RelationshipRow {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub relationship_type: String,
    pub confidence: f64,
    pub other_name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceLinkRow {
    pub id: String,
    pub workspace_id: String,
    pub relevance: Option<String>,
    pub enabled: bool,
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
            .create_profile("Commit Style", "skill", None, "internal")
            .unwrap();
        store.set_mcp_exposure(&pid, "enabled").unwrap();
        store
            .add_instruction(
                &pid,
                "style",
                &["commit", "git"],
                &["Use imperative mood"],
                100,
            )
            .unwrap();
        let matches = store
            .find_matching_instructions("create commit message")
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].profile_name, "Commit Style");
        std::fs::remove_file(&p).ok();
    }
    #[test]
    fn test_attributes() {
        let (store, p) = temp_db();
        let pid = store
            .create_profile("Me", "person", None, "private")
            .unwrap();
        store
            .set_attribute(&pid, "age", &serde_json::json!(30), None)
            .unwrap();
        let attrs = store.get_attributes(&pid).unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "age");
        std::fs::remove_file(&p).ok();
    }
    #[test]
    fn test_relationships() {
        let (store, p) = temp_db();
        let a = store
            .create_profile("A", "person", None, "private")
            .unwrap();
        let b = store
            .create_profile("B", "person", None, "private")
            .unwrap();
        store.add_relationship(&a, &b, "friend_of").unwrap();
        let rels = store.get_relationships(&a).unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].other_name, "B");
        std::fs::remove_file(&p).ok();
    }
    #[test]
    fn test_workspace_links() {
        let (store, p) = temp_db();
        let pid = store
            .create_profile("Style", "skill", None, "internal")
            .unwrap();
        store.set_mcp_exposure(&pid, "enabled").unwrap();
        store.link_workspace(&pid, "ws-1", None).unwrap();
        let profiles = store.get_profiles_for_workspace("ws-1").unwrap();
        assert_eq!(profiles.len(), 1);
        std::fs::remove_file(&p).ok();
    }
    #[test]
    fn test_archive_and_delete() {
        let (store, p) = temp_db();
        let pid = store.create_profile("X", "idea", None, "public").unwrap();
        assert_eq!(store.list_profiles().unwrap().len(), 1);
        store.archive_profile(&pid).unwrap();
        assert_eq!(store.list_profiles().unwrap().len(), 0);
        store.delete_profile(&pid).unwrap();
        assert!(store.get_profile(&pid).unwrap().is_none());
        std::fs::remove_file(&p).ok();
    }
}
