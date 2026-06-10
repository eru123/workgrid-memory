import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Workspace } from "@workgrid/schemas";

export default function WorkspacesPage() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState<Record<string, boolean>>({});

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

  const handleAddWorkspace = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select project folder",
    });

    if (!selected) return;

    const path = typeof selected === "string" ? selected : selected;
    const name = path.split("/").pop() || path;

    try {
      await invoke("add_workspace", { name, path });
      await fetchWorkspaces();
    } catch (err) {
      console.error("Failed to add workspace:", err);
    }
  };

  const handleRemoveWorkspace = async (id: string) => {
    try {
      await invoke("remove_workspace", { id });
      await fetchWorkspaces();
    } catch (err) {
      console.error("Failed to remove workspace:", err);
    }
  };

  const handleReindex = async (id: string) => {
    setActionLoading((prev) => ({ ...prev, [id]: true }));
    try {
      await invoke("reindex_workspace", { workspaceId: id });
      await fetchWorkspaces();
    } catch (err) {
      console.error("Reindex failed:", err);
    } finally {
      setActionLoading((prev) => ({ ...prev, [id]: false }));
    }
  };

  const handleCancel = async (id: string) => {
    setActionLoading((prev) => ({ ...prev, [id]: true }));
    try {
      await invoke("cancel_indexing", { workspaceId: id });
      await fetchWorkspaces();
    } catch (err) {
      console.error("Cancel failed:", err);
    } finally {
      setActionLoading((prev) => ({ ...prev, [id]: false }));
    }
  };

  const handleResume = async (id: string) => {
    setActionLoading((prev) => ({ ...prev, [id]: true }));
    try {
      await invoke("resume_indexing", { workspaceId: id });
      await fetchWorkspaces();
    } catch (err) {
      console.error("Resume failed:", err);
    } finally {
      setActionLoading((prev) => ({ ...prev, [id]: false }));
    }
  };

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

  const actionButton = (ws: Workspace) => {
    const isLoading = actionLoading[ws.id];

    if (ws.status === "indexing") {
      return (
        <button
          onClick={() => handleCancel(ws.id)}
          disabled={isLoading}
          className="px-3 py-1 rounded text-xs transition-colors cursor-pointer shrink-0"
          style={{
            background: "transparent",
            color: "var(--color-danger)",
            border: "1px solid var(--color-danger)",
          }}
        >
          {isLoading ? "..." : "Stop"}
        </button>
      );
    }

    if (ws.status === "paused") {
      return (
        <button
          onClick={() => handleResume(ws.id)}
          disabled={isLoading}
          className="px-3 py-1 rounded text-xs font-medium transition-colors cursor-pointer shrink-0"
          style={{
            background: "var(--color-accent)",
            color: "#fff",
          }}
        >
          {isLoading ? "..." : "Resume"}
        </button>
      );
    }

    // new, ready, stale, degraded, error — all get Index/Re-index
    return (
      <button
        onClick={() => handleReindex(ws.id)}
        disabled={isLoading}
        className="px-3 py-1 rounded text-xs font-medium transition-colors cursor-pointer shrink-0"
        style={{
          background: "var(--color-accent)",
          color: "#fff",
        }}
      >
        {isLoading ? "..." : ws.status === "new" ? "Index" : "Re-index"}
      </button>
    );
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold">Workspaces</h2>
        <button
          onClick={handleAddWorkspace}
          className="px-4 py-2 rounded-md text-sm font-medium transition-colors cursor-pointer"
          style={{ background: "var(--color-accent)", color: "#fff" }}
        >
          Add Workspace
        </button>
      </div>

      {loading ? (
        <p style={{ color: "var(--color-text-muted)" }}>Loading...</p>
      ) : workspaces.length === 0 ? (
        <div
          className="rounded-lg p-8 text-center"
          style={{ background: "var(--color-surface)" }}
        >
          <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
            No workspaces added yet
          </p>
          <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
            Add a project folder to start indexing your codebase.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {workspaces.map((ws) => (
            <div
              key={ws.id}
              className="rounded-lg p-4 transition-colors"
              style={{ background: "var(--color-surface)" }}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-sm font-semibold">{ws.name}</h3>
                    {statusBadge(ws.status)}
                  </div>
                  <p
                    className="text-xs truncate"
                    style={{ color: "var(--color-text-muted)" }}
                  >
                    {ws.rootPath}
                  </p>
                  {ws.gitRemote && (
                    <p className="text-xs mt-1" style={{ color: "var(--color-text-muted)" }}>
                      remote: {ws.gitRemote}
                    </p>
                  )}
                  <div
                    className="flex gap-4 mt-2 text-xs"
                    style={{ color: "var(--color-text-muted)" }}
                  >
                    <span>Added: {new Date(ws.createdAt).toLocaleDateString()}</span>
                    {ws.lastIndexedAt && (
                      <span>
                        Indexed: {new Date(ws.lastIndexedAt).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                </div>
                <div className="flex gap-2 ml-4 shrink-0">
                  {actionButton(ws)}
                  <button
                    onClick={() => handleRemoveWorkspace(ws.id)}
                    className="px-3 py-1 rounded text-xs transition-colors cursor-pointer"
                    style={{
                      background: "transparent",
                      color: "var(--color-danger)",
                      border: "1px solid var(--color-danger)",
                    }}
                  >
                    Remove
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
