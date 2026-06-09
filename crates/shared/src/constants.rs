/// Default ignore patterns for workspace scanning.
pub const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    ".git/**",
    "node_modules/**",
    "vendor/**",
    "dist/**",
    "build/**",
    "coverage/**",
    ".cache/**",
    ".next/**",
    ".nuxt/**",
    "target/**",
    "storage/logs/**",
    "*.lock",
    "*.map",
    "*.min.js",
    "*.png",
    "*.jpg",
    "*.jpeg",
    "*.gif",
    "*.webp",
    "*.ico",
    "*.pdf",
    "*.zip",
    "*.tar",
    "*.gz",
    ".env",
    ".env.*",
];

/// Supported language extensions mapped to language identifiers.
pub const LANGUAGE_EXTENSIONS: &[(&str, &str)] = &[
    ("ts", "typescript"),
    ("tsx", "tsx"),
    ("js", "javascript"),
    ("jsx", "jsx"),
    ("php", "php"),
    ("json", "json"),
    ("md", "markdown"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
    ("sql", "sql"),
    ("rs", "rust"),
    ("py", "python"),
    ("go", "go"),
    ("java", "java"),
    ("cs", "csharp"),
    ("vue", "vue"),
    ("svelte", "svelte"),
    ("css", "css"),
    ("html", "html"),
    ("toml", "toml"),
    ("env", "env"),
    ("txt", "text"),
    ("xml", "xml"),
    ("sh", "shell"),
    ("bash", "shell"),
    ("dockerfile", "dockerfile"),
];

/// Maximum file size for indexing in bytes (512 KB default).
pub const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 512 * 1024;

/// Default MCP HTTP port.
pub const DEFAULT_MCP_PORT: u16 = 3847;

/// Default MCP host binding.
pub const DEFAULT_MCP_HOST: &str = "127.0.0.1";

/// App data directory name.
pub const APP_DATA_DIR: &str = "WorkGrid Memory";
