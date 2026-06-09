export default function SearchPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Search</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          Search workspace memory
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Add and index a workspace to enable search.
        </p>
      </div>
    </div>
  );
}
