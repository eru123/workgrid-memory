export default function IndexJobsPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Index Jobs</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          No index jobs
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Index jobs will appear when workspaces are being indexed.
        </p>
      </div>
    </div>
  );
}
