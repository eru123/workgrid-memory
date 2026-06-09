interface PageHeaderProps {
  title: string;
  subtitle?: string;
}

export function PageHeader({ title, subtitle }: PageHeaderProps) {
  return (
    <div className="mb-6">
      <h2 className="text-xl font-semibold">{title}</h2>
      {subtitle && (
        <p className="text-sm mt-1" style={{ color: "var(--color-text-muted)" }}>
          {subtitle}
        </p>
      )}
    </div>
  );
}
