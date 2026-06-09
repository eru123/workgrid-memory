export default function SettingsPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Settings</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          Settings
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Configure indexing, embeddings, MCP, and profile settings.
        </p>
      </div>
    </div>
  );
}
