import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface McpStatus {
  running: boolean;
  port: number;
  tool_count: number;
}

export default function McpServerPage() {
  const [status, setStatus] = useState<McpStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<McpStatus>("get_mcp_status");
      setStatus(s);
    } catch (err) {
      console.error("Failed to get MCP status:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  const handleStart = async () => {
    setActionLoading(true);
    try {
      const s = await invoke<McpStatus>("start_mcp_server");
      setStatus(s);
    } catch (err) {
      console.error("Failed to start MCP server:", err);
    } finally {
      setActionLoading(false);
    }
  };

  const handleStop = async () => {
    setActionLoading(true);
    try {
      const s = await invoke<McpStatus>("stop_mcp_server");
      setStatus(s);
    } catch (err) {
      console.error("Failed to stop MCP server:", err);
    } finally {
      setActionLoading(false);
    }
  };

  if (loading) {
    return (
      <div>
        <h2 className="text-xl font-semibold mb-4">MCP Server</h2>
        <p style={{ color: "var(--color-text-muted)" }}>Loading...</p>
      </div>
    );
  }

  const running = status?.running ?? false;

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold">MCP Server</h2>
        <button
          onClick={running ? handleStop : handleStart}
          disabled={actionLoading}
          className="px-4 py-2 rounded-md text-sm font-medium transition-colors cursor-pointer"
          style={{
            background: running ? "var(--color-danger)" : "var(--color-accent)",
            color: "#fff",
          }}
        >
          {actionLoading ? "..." : running ? "Stop Server" : "Start Server"}
        </button>
      </div>

      <div
        className="rounded-lg p-6"
        style={{ background: "var(--color-surface)" }}
      >
        <div className="flex items-center gap-3 mb-4">
          <span
            className="inline-block w-3 h-3 rounded-full"
            style={{
              background: running ? "#4ade80" : "#8b8fa3",
            }}
          />
          <span className="text-sm font-medium">
            {running ? "Running" : "Stopped"}
          </span>
        </div>

        {status && (
          <div className="space-y-2 text-sm" style={{ color: "var(--color-text-muted)" }}>
            <p>
              <strong>Port:</strong> {status.port}
            </p>
            <p>
              <strong>Tools available:</strong> {status.tool_count}
            </p>
          </div>
        )}

        {running && (
          <div className="mt-4 p-3 rounded text-xs" style={{ background: "var(--color-surface-hover)" }}>
            <p style={{ color: "var(--color-text-muted)" }}>
              AI agents can connect via MCP at{" "}
              <code style={{ color: "var(--color-accent)" }}>
                http://localhost:{status?.port}/sse
              </code>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
