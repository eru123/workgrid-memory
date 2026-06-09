/// File scanner module — walks workspace directories, applies ignore rules,
/// computes file hashes, and detects languages.
pub mod file_scanner;

/// Ignore rules module — parses .gitignore, .workgridignore, and app-level patterns.
pub mod ignore;

/// Chunking module — splits files into meaningful chunks (symbol, structural, docs, fallback).
pub mod chunker;

/// Hashing module — computes content hashes for change detection.
pub mod hasher;
