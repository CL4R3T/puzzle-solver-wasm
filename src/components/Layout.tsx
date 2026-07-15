import { useState } from "react";
import "./Layout.css";

interface PuzzleType {
  id: string;
  name: string;
  icon: string;
}

const puzzleTypes: PuzzleType[] = [
  { id: "home", name: "首页", icon: "⌂" },
  { id: "sudoku", name: "数独求解器", icon: "⊞" },
];

interface LayoutProps {
  children: React.ReactNode;
  activePuzzle: string;
  onNavigate: (id: string) => void;
}

export default function Layout({
  children,
  activePuzzle,
  onNavigate,
}: LayoutProps) {
  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h2>纸笔求解器</h2>
        </div>
        <nav className="sidebar-nav">
          {puzzleTypes.map((puzzle) => (
            <button
              key={puzzle.id}
              className={`nav-item ${activePuzzle === puzzle.id ? "active" : ""}`}
              onClick={() => onNavigate(puzzle.id)}
            >
              <span className="nav-icon">{puzzle.icon}</span>
              <span className="nav-name">{puzzle.name}</span>
            </button>
          ))}
        </nav>
      </aside>
      <main className="main-content">{children}</main>
    </div>
  );
}

export function usePuzzleNavigation() {
  const [activePuzzle, setActivePuzzle] = useState<string>("home");
  const navigate = (id: string) => setActivePuzzle(id);
  return { activePuzzle, navigate };
}
