import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Workspace {
  id: string;
  name: string;
  rootPath: string;
  status: string;
  lastIndexedAt: string | null;
  createdAt: string;
}

export default function IndexJobsPage() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchWorkspaces = useCallback(async () => {
    try {
      const list = await invoke<Workspace[]>("list_workspaces");
      setWorkspaces(list);
    } catch (err) {
      console.error("Failed to fetch workspaces:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkspaces();
  }, [fetchWorkspaces]);

  // Poll while any workspace is indexing
  useEffect(() => {
    const hasIndexing = workspaces.some((ws) => ws.status === "indexing");
    if (!hasIndexing) return;

    const interval = setInterval(() => {
      fetchWorkspaces();
    }, 2000);

    return () => clearInterval(interval);
  }, [workspaces, fetchWorkspaces]);

  const indexing = workspaces.filter((ws) => ws.status === "indexing");
  const completed = workspaces.filter((ws) => ws.status === "ready");
  const paused = workspaces.filter((ws) => ws.status === "paused");
  const other = workspaces.filter(
    (ws) => !["indexing", "ready", "paused"].includes(ws.status)
  );

  const statusBadge = (status: string) => {
    const colors: Record<string, string> = {
      new: "#8b8fa3",
      indexing: "#ffb347",
      ready: "#4ade80",
      degraded: "#ff6b6b",
      stale: "#ffb347",
      error: "#ff6b6b",
      paused: "#8b8fa3",
    };
    return (
      <span
        className="px-2 py-0.5 rounded text-xs font-medium"
        style={{
          background: `${colors[status] || "#8b8fa3"}20`,
          color: colors[status] || "#8b8fa3",
        }}
      >
        {status}
      </span>
    );
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Index Jobs</h2>

      {loading ? (
        <p style={{ color: "var(--color-text-muted)" }}>Loading...</p>
      ) : workspaces.length === 0 ? (
        <div
          className="rounded-lg p-8 text-center"
          style={{ background: "var(--color-surface)" }}
        >
          <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
            No workspaces added
          </p>
          <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
            Add a workspace to begin indexing.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {/* In-progress */}
          {indexing.length > 0 && (
            <div>
              <h3
                className="text-sm font-semibold mb-2"
                style={{ color: "#ffb347" }}
              >
                In Progress ({indexing.length})
              </h3>
              <div className="space-y-2">
                {indexing.map((ws) => (
                  <div
                    key={ws.id}
                    className="rounded-lg p-3 flex items-center justify-between"
                    style={{ background: "var(--color-surface)" }}
                  >
                    <div>
                      <span className="text-sm font-medium">{ws.name}</span>
                      <span className="text-xs ml-2" style={{ color: "var(--color-text-muted)" }}>
                        {ws.rootPath}
                      </span>
                    </div>
                    {statusBadge(ws.status)}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Paused */}
          {paused.length > 0 && (
            <div>
              <h3
                className="text-sm font-semibold mb-2"
                style={{ color: "var(--color-text-muted)" }}
              >
                Paused ({paused.length})
              </h3>
              <div className="space-y-2">
                {paused.map((ws) => (
                  <div
                    key={ws.id}
                    className="rounded-lg p-3 flex items-center justify-between"
                    style={{ background: "var(--color-surface)" }}
                  >
                    <div>
                      <span className="text-sm font-medium">{ws.name}</span>
                      <span className="text-xs ml-2" style={{ color: "var(--color-text-muted)" }}>
                        {ws.rootPath}
                      </span>
                    </div>
                    {statusBadge(ws.status)}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Completed */}
          {completed.length > 0 && (
            <div>
              <h3
                className="text-sm font-semibold mb-2"
                style={{ color: "#4ade80" }}
              >
                Completed ({completed.length})
              </h3>
              <div className="space-y-2">
                {completed.map((ws) => (
                  <div
                    key={ws.id}
                    className="rounded-lg p-3 flex items-center justify-between"
                    style={{ background: "var(--color-surface)" }}
                  >
                    <div>
                      <span className="text-sm font-medium">{ws.name}</span>
                      <span className="text-xs ml-2" style={{ color: "var(--color-text-muted)" }}>
                        {ws.rootPath}
                      </span>
                      {ws.lastIndexedAt && (
                        <span className="text-xs ml-2" style={{ color: "var(--color-text-muted)" }}>
                          — {new Date(ws.lastIndexedAt).toLocaleString()}
                        </span>
                      )}
                    </div>
                    {statusBadge(ws.status)}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Other states */}
          {other.length > 0 && (
            <div>
              <h3
                className="text-sm font-semibold mb-2"
                style={{ color: "var(--color-text-muted)" }}
              >
                Other ({other.length})
              </h3>
              <div className="space-y-2">
                {other.map((ws) => (
                  <div
                    key={ws.id}
                    className="rounded-lg p-3 flex items-center justify-between"
                    style={{ background: "var(--color-surface)" }}
                  >
                    <div>
                      <span className="text-sm font-medium">{ws.name}</span>
                      <span className="text-xs ml-2" style={{ color: "var(--color-text-muted)" }}>
                        {ws.rootPath}
                      </span>
                    </div>
                    {statusBadge(ws.status)}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
