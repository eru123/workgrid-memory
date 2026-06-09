export default function ProfilesPage() {
  return (
    <div>
      <h2 className="text-xl font-semibold mb-4">Profiles</h2>
      <div
        className="rounded-lg p-8 text-center"
        style={{ background: "var(--color-surface)", borderColor: "var(--color-border)" }}
      >
        <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
          No profiles created yet
        </p>
        <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Create profiles for people, pets, places, workflows, and more.
        </p>
      </div>
    </div>
  );
}
