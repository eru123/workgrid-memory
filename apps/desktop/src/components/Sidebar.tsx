import { NavLink } from "react-router-dom";

const navItems = [
  { to: "/workspaces", label: "Workspaces", icon: "📁" },
  { to: "/profiles", label: "Profiles", icon: "👤" },
  { to: "/search", label: "Search", icon: "🔍" },
  { to: "/context-packs", label: "Context Packs", icon: "📦" },
  { to: "/mcp-server", label: "MCP Server", icon: "🔌" },
  { to: "/index-jobs", label: "Index Jobs", icon: "⚙️" },
  { to: "/settings", label: "Settings", icon: "⚡" },
  { to: "/logs", label: "Logs", icon: "📋" },
];

export default function Sidebar() {
  return (
    <aside
      className="flex flex-col w-56 border-r shrink-0"
      style={{
        background: "var(--color-surface)",
        borderColor: "var(--color-border)",
      }}
    >
      <div className="p-4 border-b" style={{ borderColor: "var(--color-border)" }}>
        <h1 className="text-sm font-semibold tracking-wide" style={{ color: "var(--color-accent)" }}>
          WorkGrid Memory
        </h1>
        <p className="text-xs mt-1" style={{ color: "var(--color-text-muted)" }}>
          v0.1.0
        </p>
      </div>
      <nav className="flex-1 py-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-2 text-sm transition-colors ${
                isActive
                  ? "font-medium"
                  : "hover:bg-opacity-50"
              }`
            }
            style={({ isActive }) => ({
              background: isActive ? "var(--color-surface-hover)" : "transparent",
              color: isActive ? "var(--color-text)" : "var(--color-text-muted)",
              borderLeft: isActive
                ? "3px solid var(--color-accent)"
                : "3px solid transparent",
            })}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>
      <div
        className="p-3 border-t text-xs"
        style={{ borderColor: "var(--color-border)", color: "var(--color-text-muted)" }}
      >
        Local-first · v0.1.0
      </div>
    </aside>
  );
}
