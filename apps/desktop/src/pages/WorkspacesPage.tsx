export default function WorkspacesPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Workspaces</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          No workspaces added yet
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Add a project folder to start indexing your codebase.
        </p>
      </div>
    </div>
  );
}
