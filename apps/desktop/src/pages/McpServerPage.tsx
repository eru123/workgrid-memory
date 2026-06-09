export default function McpServerPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">MCP Server</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          MCP server is stopped
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Start the MCP server to expose workspace and profile context to AI agents.
        </p>
      </div>
    </div>
  );
}
