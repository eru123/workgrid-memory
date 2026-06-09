export default function ContextPacksPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Context Packs</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          No context packs generated
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Context packs combine workspace evidence with relevant profile context.
        </p>
      </div>
    </div>
  );
}
