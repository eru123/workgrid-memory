export default function LogsPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Logs</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          No log entries
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Application logs will appear here.
        </p>
      </div>
    </div>
  );
}
