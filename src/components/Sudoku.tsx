import { useCallback, useEffect, useState } from "react";
import { SudokuGrid, emptyBoard, type SelectionMode } from "./SudokuGrid";
import { ConstraintToolbar } from "./ConstraintToolbar";
import { ConstraintSidebar } from "./ConstraintSidebar";
import { solveSudoku, validateSudoku, type SolveParams } from "../api/sudoku";
import type { ConstraintInstance } from "../constraints/definitions";
import {
  getNextUnusedColor,
  getDefaultTypeForCategory,
} from "../constraints/definitions";
import "./Sudoku.css";

type Board = number[][];
interface BlockShape {
  rows: number;
  cols: number;
}
type MessageType = "success" | "error" | "";

function isAdjacent4(r: number, c: number, cells: [number, number][]): boolean {
  return cells.some(([cr, cc]) => Math.abs(r - cr) + Math.abs(c - cc) === 1);
}

function isAdjacent8(r: number, c: number, target: [number, number]): boolean {
  return Math.max(Math.abs(r - target[0]), Math.abs(c - target[1])) === 1;
}

function hasCell(cells: [number, number][], r: number, c: number): boolean {
  return cells.some(([cr, cc]) => cr === r && cc === c);
}

function isConnected4(cells: [number, number][]): boolean {
  if (cells.length <= 1) return true;
  const set = new Set(cells.map(([r, c]) => `${r},${c}`));
  const visited = new Set<string>();
  const queue: [number, number][] = [cells[0]];
  visited.add(`${cells[0][0]},${cells[0][1]}`);
  while (queue.length > 0) {
    const [r, c] = queue.shift()!;
    for (const [dr, dc] of [
      [-1, 0],
      [1, 0],
      [0, -1],
      [0, 1],
    ]) {
      const nr = r + dr,
        nc = c + dc;
      const key = `${nr},${nc}`;
      if (set.has(key) && !visited.has(key)) {
        visited.add(key);
        queue.push([nr, nc]);
      }
    }
  }
  return visited.size === cells.length;
}

export default function Sudoku() {
  const [board, setBoard] = useState<Board>(emptyBoard(9));
  const [appliedSideLength, setAppliedSideLength] = useState<number>(9);
  const [appliedShape, setAppliedShape] = useState<BlockShape>({
    rows: 3,
    cols: 3,
  });
  const [message, setMessage] = useState<string>("");
  const [messageType, setMessageType] = useState<MessageType>("");
  const [loading, setLoading] = useState<boolean>(false);

  // settings dialog state
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const [sideLength, setSideLength] = useState<number>(9);
  const [shapeOptions, setShapeOptions] = useState<BlockShape[]>([]);
  const [selectedShape, setSelectedShape] = useState<BlockShape | null>(null);
  const [sideLengthError, setSideLengthError] = useState<string>("");

  // constraint state
  const [constraints, setConstraints] = useState<ConstraintInstance[]>([]);
  const [activeConstraintId, setActiveConstraintId] = useState<string | null>(
    null,
  );
  const [selectionMode, setSelectionMode] = useState<SelectionMode>("none");
  const [currentCells, setCurrentCells] = useState<[number, number][]>([]);
  const [showSidebar, setShowSidebar] = useState<boolean>(false);
  const [solutionBoard, setSolutionBoard] = useState<Board | null>(null);

  const diagonalsEnabled = constraints.some(
    (c) => c.constraintType === "diagonals",
  );

  // ---- settings helpers ----

  const isPrime = (n: number): boolean => {
    if (n < 2) return false;
    for (let i = 2; i * i <= n; i++) {
      if (n % i === 0) return false;
    }
    return true;
  };

  const computeShapes = (n: number): BlockShape[] => {
    const opts: BlockShape[] = [];
    for (let i = 1; i <= n; i++) {
      if (n % i === 0) {
        const rows = i;
        const cols = n / i;
        if (rows === 1 || cols === 1) continue;
        opts.push({ rows, cols });
      }
    }
    return opts;
  };

  const applySettings = (): void => {
    if (sideLengthError || !selectedShape) return;
    const size = sideLength;
    if (size === appliedSideLength) {
      setAppliedShape(selectedShape);
      setShowSettings(false);
      return;
    }

    setBoard(
      Array(size)
        .fill(null)
        .map(() => Array(size).fill(0)),
    );
    setAppliedSideLength(size);
    setAppliedShape(selectedShape);
    setConstraints([]);
    setActiveConstraintId(null);
    setCurrentCells([]);
    setSelectionMode("none");
    setSolutionBoard(null);
    setShowSettings(false);
  };

  const handleSideLengthChange = (
    e: React.ChangeEvent<HTMLInputElement>,
  ): void => {
    const v = parseInt(e.target.value, 10);
    if (isNaN(v)) {
      setSideLength(e.target.value as any);
      setSideLengthError("");
      return;
    }

    setSideLength(v);

    if (v < 4 || v > 16) {
      setSideLengthError("边长必须在 4 到 16 之间");
      return;
    }
    if (isPrime(v)) {
      setSideLengthError("边长不能为质数");
      return;
    }

    setSideLengthError("");
    const opts = computeShapes(v);
    setShapeOptions(opts);
    if (
      !opts.some(
        (o) =>
          o.rows === (selectedShape?.rows || 0) &&
          o.cols === (selectedShape?.cols || 0),
      )
    ) {
      setSelectedShape(opts[0] || null);
    }
  };

  useEffect(() => {
    const opts = computeShapes(sideLength);
    setShapeOptions(opts);
    setSelectedShape(opts[0] || null);
  }, []);

  // ---- selection handlers ----

  const handleCellMouseDown = useCallback(
    (r: number, c: number) => {
      if (selectionMode === "none") return;
      if (selectionMode === "region") {
        setCurrentCells([[r, c]]);
      }
      if (selectionMode === "path") {
        setCurrentCells((prev) => (prev.length === 0 ? [[r, c]] : prev));
      }
    },
    [selectionMode],
  );

  const handleCellMouseEnter = useCallback(
    (r: number, c: number) => {
      if (selectionMode === "none") return;
      if (selectionMode === "region") {
        setCurrentCells((prev) => {
          if (prev.length === 0) return prev;
          if (!isAdjacent4(r, c, prev)) return prev;
          if (hasCell(prev, r, c)) return prev;
          return [...prev, [r, c]];
        });
      }
      if (selectionMode === "path") {
        setCurrentCells((prev) => {
          if (prev.length === 0) return prev;
          // if already in path: undo if it's the second-to-last cell, otherwise ignore
          if (hasCell(prev, r, c)) {
            if (
              prev.length >= 2 &&
              r === prev[prev.length - 2][0] &&
              c === prev[prev.length - 2][1]
            ) {
              return prev.slice(0, -1);
            }
            return prev;
          }
          const last = prev[prev.length - 1];
          if (!isAdjacent8(r, c, last)) return prev;
          return [...prev, [r, c]];
        });
      }
    },
    [selectionMode],
  );

  const handleCellMouseUp = useCallback(() => {
    if (selectionMode === "none" || currentCells.length === 0) return;

    if (selectionMode === "path" && currentCells.length < 2) {
      setCurrentCells([]);
      return;
    }
    if (selectionMode === "region" && !isConnected4(currentCells)) {
      setMessage("选区不连通，请重试");
      setMessageType("error");
      setCurrentCells([]);
      return;
    }

    const defaultType = getDefaultTypeForCategory(
      selectionMode === "path" ? "path" : "region",
    );
    const draft: ConstraintInstance = {
      id: crypto.randomUUID(),
      constraintType: defaultType,
      cells: [...currentCells],
      params: defaultType === "cages" ? { sum: 0 } : {},
      color: getNextUnusedColor(constraints),
    };
    setConstraints((prev) => [...prev, draft]);
    setActiveConstraintId(draft.id);
    setCurrentCells([]);
    setSelectionMode("none");
    setShowSidebar(true);
  }, [selectionMode, currentCells, constraints]);

  // ---- constraint operations ----

  const updateConstraint = useCallback(
    (id: string, patch: Partial<ConstraintInstance>) => {
      setConstraints((prev) =>
        prev.map((c) => (c.id === id ? { ...c, ...patch } : c)),
      );
    },
    [],
  );

  const deleteConstraint = useCallback(
    (id: string) => {
      setConstraints((prev) => prev.filter((c) => c.id !== id));
      if (activeConstraintId === id) setActiveConstraintId(null);
    },
    [activeConstraintId],
  );

  const toggleDiagonals = useCallback(() => {
    setConstraints((prev) => {
      const existing = prev.find((c) => c.constraintType === "diagonals");
      if (existing) return prev.filter((c) => c.id !== existing.id);
      return [
        ...prev,
        {
          id: crypto.randomUUID(),
          constraintType: "diagonals",
          cells: [],
          params: {},
          color: "#60a5fa",
        },
      ];
    });
  }, []);

  const selectConstraint = useCallback((id: string | null) => {
    setActiveConstraintId(id);
    setSelectionMode("none");
    setCurrentCells([]);
  }, []);

  // ---- API params builder ----

  const buildSolveParams = useCallback((): SolveParams => {
    const p: SolveParams = { boxShape: [appliedShape.rows, appliedShape.cols] };
    if (diagonalsEnabled) p.diagonals = true;
    const cages = constraints
      .filter((c) => c.constraintType === "cages" && c.cells.length > 0)
      .map((c) => ({ cells: c.cells, sum: c.params.sum as number }));
    if (cages.length > 0) p.cages = cages;
    const thermos = constraints
      .filter((c) => c.constraintType === "thermometer" && c.cells.length >= 2)
      .map((c) => c.cells);
    if (thermos.length > 0) p.thermos = thermos;
    const palindromes = constraints
      .filter((c) => c.constraintType === "palindrome" && c.cells.length >= 2)
      .map((c) => c.cells);
    if (palindromes.length > 0) p.palindromes = palindromes;
    return p;
  }, [appliedShape, diagonalsEnabled, constraints]);

  // ---- clear / validate / solve ----

  const handleClear = useCallback((): void => {
    setBoard(emptyBoard(appliedSideLength));
    setConstraints([]);
    setActiveConstraintId(null);
    setCurrentCells([]);
    setSelectionMode("none");
    setSolutionBoard(null);
    setMessage("");
    setMessageType("");
  }, [appliedSideLength]);

  const handleValidate = useCallback(async (): Promise<void> => {
    setMessage("");
    setMessageType("");
    setLoading(true);
    try {
      const data = await validateSudoku(board, buildSolveParams());
      setMessage(data.message);
      setMessageType(data.valid ? "success" : "error");
    } catch (err) {
      setMessage((err as Error).message || "求解失败，请重试");
      setMessageType("error");
    } finally {
      setLoading(false);
    }
  }, [board, buildSolveParams]);

  const handleSolve = useCallback(async (): Promise<void> => {
    setMessage("");
    setMessageType("");
    setLoading(true);
    try {
      const data = await solveSudoku(board, buildSolveParams());
      setMessage(data.message);
      setMessageType(data.success ? "success" : "error");
      if (data.success && data.solution) {
        setSolutionBoard(data.solution);
      }
    } catch (err) {
      setMessage((err as Error).message || "求解失败，请重试");
      setMessageType("error");
    } finally {
      setLoading(false);
    }
  }, [board, buildSolveParams]);

  // ---- render ----

  return (
    <div className="app">
      <header className="app-header">
        <h1>数独求解器</h1>
        <p className="subtitle">填写部分数字后点击「校验」或「求解」</p>
      </header>

      <main className="app-main">
        <ConstraintToolbar
          selectionMode={selectionMode}
          onSelectMode={(mode) => {
            setSelectionMode(mode);
            setCurrentCells([]);
          }}
          diagonalsEnabled={diagonalsEnabled}
          onToggleDiagonals={toggleDiagonals}
          constraintCount={constraints.length}
          onToggleSidebar={() => setShowSidebar((v) => !v)}
          disabled={loading}
        />

        {selectionMode !== "none" && (
          <div className="constraint-toolbar">
            <span
              style={{
                fontSize: "0.85rem",
                color: "var(--text-muted, #94a3b8)",
              }}
            >
              {selectionMode === "region"
                ? "按住鼠标左键选择区域，松手后创建选区。"
                : "按住鼠标左键构建路径，松手后创建选区。"}
              {" · "}
              已选 {currentCells.length} 格
            </span>
          </div>
        )}

        <SudokuGrid
          board={board}
          onChange={setBoard}
          readOnly={false}
          blockRows={appliedShape.rows}
          blockCols={appliedShape.cols}
          constraints={constraints}
          activeConstraintId={activeConstraintId}
          selectionMode={selectionMode}
          currentCells={currentCells}
          previewColor={getNextUnusedColor(constraints)}
          onCellMouseDown={handleCellMouseDown}
          onCellMouseEnter={handleCellMouseEnter}
          onCellMouseUp={handleCellMouseUp}
        />

        <div className="actions">
          <button
            type="button"
            onClick={() => setShowSettings(true)}
            disabled={loading}
          >
            设置
          </button>
          <button type="button" onClick={handleClear} disabled={loading}>
            清空
          </button>
          <button type="button" onClick={handleValidate} disabled={loading}>
            {loading ? "校验中…" : "校验"}
          </button>
          <button type="button" onClick={handleSolve} disabled={loading}>
            {loading ? "求解中…" : "求解"}
          </button>
        </div>

        {message && (
          <div className={`message message-${messageType}`} role="status">
            {message}
          </div>
        )}

        {showSettings && (
          <div
            className="modal-backdrop"
            onClick={() => setShowSettings(false)}
          >
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <h2>设置数独盘面</h2>

              <div className="modal-row">
                <label htmlFor="side-length">边长</label>
                <input
                  id="side-length"
                  type="number"
                  min="4"
                  max="16"
                  value={sideLength}
                  onChange={handleSideLengthChange}
                />
              </div>
              {sideLengthError && (
                <div
                  className="modal-error"
                  role="alert"
                  style={{ color: "red", fontSize: "0.9rem" }}
                >
                  {sideLengthError}
                </div>
              )}

              <div className="modal-row">
                <label htmlFor="shape-select">宫的形状</label>
                <select
                  id="shape-select"
                  value={
                    selectedShape
                      ? `${selectedShape.rows}x${selectedShape.cols}`
                      : ""
                  }
                  onChange={(e: React.ChangeEvent<HTMLSelectElement>) => {
                    const [r, cCol] = e.target.value.split("x").map(Number);
                    setSelectedShape({ rows: r, cols: cCol });
                  }}
                >
                  {shapeOptions.map((opt) => (
                    <option
                      key={`${opt.rows}x${opt.cols}`}
                      value={`${opt.rows}x${opt.cols}`}
                    >
                      {opt.rows} × {opt.cols}
                    </option>
                  ))}
                </select>
              </div>

              <div className="modal-actions">
                <button type="button" onClick={() => setShowSettings(false)}>
                  取消
                </button>
                <button
                  type="button"
                  onClick={applySettings}
                  disabled={!!sideLengthError || !selectedShape}
                >
                  确定
                </button>
              </div>
            </div>
          </div>
        )}

        {solutionBoard && (
          <div
            className="modal-backdrop"
            onClick={() => setSolutionBoard(null)}
          >
            <div
              className="modal"
              onClick={(e) => e.stopPropagation()}
              style={{ maxWidth: "36rem" }}
            >
              <h2>求解结果</h2>
              <div
                style={{
                  display: "flex",
                  justifyContent: "center",
                  margin: "1rem 0",
                }}
              >
                <SudokuGrid
                  board={solutionBoard}
                  onChange={() => {}}
                  readOnly={true}
                  blockRows={appliedShape.rows}
                  blockCols={appliedShape.cols}
                  constraints={constraints}
                  solvedMask={board.map((row) => row.map((v) => v === 0))}
                />
              </div>
              <div className="modal-actions">
                <button type="button" onClick={() => setSolutionBoard(null)}>
                  关闭
                </button>
              </div>
            </div>
          </div>
        )}

        {showSidebar && (
          <ConstraintSidebar
            constraints={constraints}
            activeConstraintId={activeConstraintId}
            onSelect={selectConstraint}
            onUpdate={updateConstraint}
            onDelete={deleteConstraint}
            onClose={() => setShowSidebar(false)}
          />
        )}
      </main>
    </div>
  );
}
