import { Routes, Route, Navigate } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import WorkspacesPage from "./pages/WorkspacesPage";
import ProfilesPage from "./pages/ProfilesPage";
import SearchPage from "./pages/SearchPage";
import ContextPacksPage from "./pages/ContextPacksPage";
import McpServerPage from "./pages/McpServerPage";
import IndexJobsPage from "./pages/IndexJobsPage";
import SettingsPage from "./pages/SettingsPage";
import LogsPage from "./pages/LogsPage";

export default function App() {
  return (
    <div className="flex h-full">
      <Sidebar />
      <main className="flex-1 overflow-auto p-6">
        <Routes>
          <Route path="/" element={<Navigate to="/workspaces" replace />} />
          <Route path="/workspaces" element={<WorkspacesPage />} />
          <Route path="/profiles" element={<ProfilesPage />} />
          <Route path="/search" element={<SearchPage />} />
          <Route path="/context-packs" element={<ContextPacksPage />} />
          <Route path="/mcp-server" element={<McpServerPage />} />
          <Route path="/index-jobs" element={<IndexJobsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/logs" element={<LogsPage />} />
        </Routes>
      </main>
    </div>
  );
}
