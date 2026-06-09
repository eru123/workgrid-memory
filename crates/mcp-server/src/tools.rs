use serde::{Deserialize, Serialize};

/// MCP JSON-RPC message types.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "tools/list")]
    ToolsList,
    #[serde(rename = "tools/call")]
    ToolsCall(ToolCallParams),
    #[serde(rename = "initialize")]
    Initialize(InitializeParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: serde_json::Value,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Returns all MCP tool definitions.
pub fn get_tools() -> Vec<Tool> {
    vec![
        search_workspace_tool(),
        get_file_context_tool(),
        explain_symbol_tool(),
        find_references_tool(),
        get_related_files_tool(),
        get_workspace_map_tool(),
        verify_claim_tool(),
        search_profiles_tool(),
        get_profile_context_tool(),
        get_relevant_profiles_tool(),
        build_context_pack_tool(),
    ]
}

fn search_workspace_tool() -> Tool {
    Tool {
        name: "search_workspace".into(),
        description: "Search workspace memory using hybrid retrieval (keyword + vector)".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "workspace_id": { "type": "string", "description": "Workspace ID" },
                "top_k": { "type": "integer", "default": 8 },
                "include_snippets": { "type": "boolean", "default": true }
            },
            "required": ["query", "workspace_id"]
        }),
    }
}

fn get_file_context_tool() -> Tool {
    Tool {
        name: "get_file_context".into(),
        description: "Get context for a specific file including symbols, imports, and related files".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "file_path": { "type": "string" }
            },
            "required": ["workspace_id", "file_path"]
        }),
    }
}

fn explain_symbol_tool() -> Tool {
    Tool {
        name: "explain_symbol".into(),
        description: "Explain a symbol (function, class, method, etc.) with definition and references".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "symbol_name": { "type": "string" }
            },
            "required": ["workspace_id", "symbol_name"]
        }),
    }
}

fn find_references_tool() -> Tool {
    Tool {
        name: "find_references".into(),
        description: "Find references to a symbol using indexed graph data".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "symbol_name": { "type": "string" }
            },
            "required": ["workspace_id", "symbol_name"]
        }),
    }
}

fn get_related_files_tool() -> Tool {
    Tool {
        name: "get_related_files".into(),
        description: "Get files related by imports, routes, database usage, or symbol relationships".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "file_path": { "type": "string" }
            },
            "required": ["workspace_id", "file_path"]
        }),
    }
}

fn get_workspace_map_tool() -> Tool {
    Tool {
        name: "get_workspace_map".into(),
        description: "Return a compact architecture map of the workspace".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" }
            },
            "required": ["workspace_id"]
        }),
    }
}

fn verify_claim_tool() -> Tool {
    Tool {
        name: "verify_claim".into(),
        description: "Check whether a statement is supported by indexed project evidence".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "claim": { "type": "string", "description": "The claim to verify" }
            },
            "required": ["workspace_id", "claim"]
        }),
    }
}

fn search_profiles_tool() -> Tool {
    Tool {
        name: "search_profiles".into(),
        description: "Search global profiles by name, type, attributes, and instructions".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "profile_types": { "type": "array", "items": { "type": "string" } },
                "include_sensitive": { "type": "boolean", "default": false }
            },
            "required": ["query"]
        }),
    }
}

fn get_profile_context_tool() -> Tool {
    Tool {
        name: "get_profile_context".into(),
        description: "Get a specific profile's context with sensitivity filtering".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "profile_id": { "type": "string" }
            },
            "required": ["profile_id"]
        }),
    }
}

fn get_relevant_profiles_tool() -> Tool {
    Tool {
        name: "get_relevant_profiles".into(),
        description: "Get profiles relevant to a task based on trigger terms and workspace links".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "task": { "type": "string", "description": "The task description" }
            },
            "required": ["workspace_id", "task"]
        }),
    }
}

fn build_context_pack_tool() -> Tool {
    Tool {
        name: "build_context_pack".into(),
        description: "Build a context pack with separated workspace evidence and profile context".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "workspace_id": { "type": "string" },
                "task": { "type": "string" },
                "include_workspace": { "type": "boolean", "default": true },
                "include_profiles": { "type": "boolean", "default": true }
            },
            "required": ["workspace_id", "task"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_defined() {
        let tools = get_tools();
        assert_eq!(tools.len(), 11);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search_workspace"));
        assert!(names.contains(&"verify_claim"));
        assert!(names.contains(&"search_profiles"));
        assert!(names.contains(&"build_context_pack"));
    }

    #[test]
    fn test_tool_schemas_valid() {
        for tool in get_tools() {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.input_schema.is_object());
        }
    }
}
