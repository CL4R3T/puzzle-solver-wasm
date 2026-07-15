import "./Layout.css";

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h2>纸笔求解器</h2>
        </div>
        <nav className="sidebar-nav">
          <button className="nav-item active">
            <span className="nav-icon">⊞</span>
            <span className="nav-name">数独求解器</span>
          </button>
        </nav>
      </aside>
      <main className="main-content">{children}</main>
    </div>
  );
}
