import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Workspace {
  id: string;
  name: string;
  rootPath: string;
  status: string;
}

interface SearchResultItem {
  chunk_id: string;
  file_path: string;
  content: string;
  start_line: number;
  end_line: number;
  score: number;
  match_reason: string;
}

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResultItem[]>([]);
  const [searching, setSearching] = useState(false);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [selectedWs, setSelectedWs] = useState<string>("");
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadWorkspaces = useCallback(async () => {
    try {
      const list = await invoke<Workspace[]>("list_workspaces");
      setWorkspaces(list);
      const ready = list.find((w) => w.status === "ready");
      if (ready) setSelectedWs(ready.id);
    } catch (err) {
      console.error("Failed to load workspaces:", err);
    }
  }, []);

  // Load workspaces on mount
  useEffect(() => {
    loadWorkspaces();
  }, [loadWorkspaces]);

  const handleSearch = async () => {
    if (!query.trim() || !selectedWs) return;

    setSearching(true);
    setError(null);
    setSearched(true);

    try {
      const res = await invoke<SearchResultItem[]>("search_workspace", {
        workspaceId: selectedWs,
        query: query.trim(),
        topK: 20,
      });
      setResults(res);
    } catch (err) {
      setError(String(err));
      setResults([]);
    } finally {
      setSearching(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSearch();
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Search</h2>

      {/* Search bar */}
      <div className="flex gap-2 mb-4">
        <select
          value={selectedWs}
          onChange={(e) => setSelectedWs(e.target.value)}
          className="px-3 py-2 rounded-md text-sm border cursor-pointer"
          style={{
            background: "var(--color-surface)",
            color: "var(--color-text)",
            borderColor: "var(--color-border)",
          }}
        >
          <option value="">Select workspace...</option>
          {workspaces.map((ws) => (
            <option key={ws.id} value={ws.id}>
              {ws.name} ({ws.status})
            </option>
          ))}
        </select>

        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Search code, symbols, docs..."
          className="flex-1 px-3 py-2 rounded-md text-sm border"
          style={{
            background: "var(--color-surface)",
            color: "var(--color-text)",
            borderColor: "var(--color-border)",
          }}
        />

        <button
          onClick={handleSearch}
          disabled={searching || !query.trim() || !selectedWs}
          className="px-4 py-2 rounded-md text-sm font-medium transition-colors cursor-pointer"
          style={{
            background: "var(--color-accent)",
            color: "#fff",
            opacity: searching || !query.trim() || !selectedWs ? 0.6 : 1,
          }}
        >
          {searching ? "..." : "Search"}
        </button>
      </div>

      {/* Results area */}
      {!searched ? (
        <div
          className="rounded-lg p-8 text-center"
          style={{ background: "var(--color-surface)" }}
        >
          <p style={{ color: "var(--color-text-muted)" }}>
            Enter a query and select a workspace to search.
          </p>
        </div>
      ) : error ? (
        <div
          className="rounded-lg p-4 text-sm"
          style={{
            background: "var(--color-surface)",
            color: "var(--color-danger)",
          }}
        >
          {error}
        </div>
      ) : results.length === 0 ? (
        <div
          className="rounded-lg p-8 text-center"
          style={{ background: "var(--color-surface)" }}
        >
          <p style={{ color: "var(--color-text-muted)" }}>
            No results found for "{query}".
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          <p className="text-xs mb-2" style={{ color: "var(--color-text-muted)" }}>
            {results.length} result{results.length !== 1 ? "s" : ""}
          </p>
          {results.map((r) => (
            <div
              key={r.chunk_id}
              className="rounded-lg p-3 text-sm"
              style={{ background: "var(--color-surface)" }}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-mono" style={{ color: "var(--color-accent)" }}>
                  {r.file_path}:{r.start_line}-{r.end_line}
                </span>
                <span
                  className="text-xs px-1.5 py-0.5 rounded"
                  style={{
                    background: "var(--color-surface-hover)",
                    color: "var(--color-text-muted)",
                  }}
                >
                  {r.match_reason}
                </span>
              </div>
              <pre
                className="text-xs whitespace-pre-wrap overflow-x-auto mt-1"
                style={{ color: "var(--color-text)", maxHeight: "120px" }}
              >
                {r.content.slice(0, 500)}
                {r.content.length > 500 ? "..." : ""}
              </pre>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
