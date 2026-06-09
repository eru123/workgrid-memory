interface EmptyStateProps {
  title: string;
  description: string;
}

export function EmptyState({ title, description }: EmptyStateProps) {
  return (
    <div
      className="rounded-lg p-8 text-center"
      style={{ background: "var(--color-surface)" }}
    >
      <p className="text-lg mb-2" style={{ color: "var(--color-text-muted)" }}>
        {title}
      </p>
      <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
        {description}
      </p>
    </div>
  );
}
