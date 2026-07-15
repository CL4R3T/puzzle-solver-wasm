import { useCallback, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { SudokuCell, type CellOverlay } from './SudokuCell'
import type { ConstraintInstance } from '../constraints/definitions'
import { getCategoryForType } from '../constraints/definitions'

type Board = number[][]

export type SelectionMode = 'none' | 'region' | 'path'

const emptyBoard = (size = 9): Board => Array(size).fill(null).map(() => Array(size).fill(0))

interface PathLine {
  cells: [number, number][]
  color: string
  isActive: boolean
  isPreview: boolean
}

interface SudokuGridProps {
  board: Board
  onChange: (board: Board) => void
  readOnly?: boolean
  blockRows?: number
  blockCols?: number
  constraints?: ConstraintInstance[]
  activeConstraintId?: string | null
  selectionMode?: SelectionMode
  currentCells?: [number, number][]
  previewColor?: string
  solvedMask?: boolean[][] | null
  onCellMouseDown?: (r: number, c: number) => void
  onCellMouseEnter?: (r: number, c: number) => void
  onCellMouseUp?: () => void
}

export function SudokuGrid({
  board,
  onChange,
  readOnly = false,
  blockRows = 3,
  blockCols = 3,
  constraints = [],
  activeConstraintId = null,
  selectionMode = 'none',
  currentCells = [],
  previewColor = '#94a3b8',
  solvedMask = null,
  onCellMouseDown,
  onCellMouseEnter,
  onCellMouseUp,
}: SudokuGridProps) {
  const gridRef = useRef<HTMLDivElement>(null)
  const [cellCenters, setCellCenters] = useState<Map<string, { cx: number; cy: number }>>(new Map())
  const gridSizeRef = useRef({ w: 0, h: 0 })
  const pathThresholdRef = useRef(12)
  const gridReadOnly = readOnly || selectionMode !== 'none'

  const handleCellChange = useCallback(
    (row: number, col: number, value: number) => {
      const next = board.map((r, i) =>
        i === row ? r.map((v, j) => (j === col ? value : v)) : r
      )
      onChange(next)
    },
    [board, onChange]
  )

  // measure cell positions relative to grid
  const recalcCenters = useCallback(() => {
    const grid = gridRef.current
    if (!grid) return
    const gridRect = grid.getBoundingClientRect()
    gridSizeRef.current = { w: grid.clientWidth, h: grid.clientHeight }
    const wraps = grid.querySelectorAll('.sudoku-cell-wrap')
    const map = new Map<string, { cx: number; cy: number }>()
    wraps.forEach((el) => {
      const r = (el as HTMLElement).dataset.row
      const c = (el as HTMLElement).dataset.col
      if (r === undefined || c === undefined) return
      const rect = el.getBoundingClientRect()
      map.set(`${r},${c}`, {
        cx: rect.left - gridRect.left + rect.width / 2 - grid.clientLeft,
        cy: rect.top - gridRect.top + rect.height / 2 - grid.clientTop,
      })
    })
    setCellCenters(map)
  }, [])

  useLayoutEffect(() => {
    recalcCenters()
    const obs = new ResizeObserver(recalcCenters)
    if (gridRef.current) obs.observe(gridRef.current)
    return () => obs.disconnect()
  }, [board.length, blockRows, blockCols, recalcCenters])

  // derive path threshold from measured cell spacing
  useLayoutEffect(() => {
    if (cellCenters.size < 2) return
    const c00 = cellCenters.get('0,0')
    const c01 = cellCenters.get('0,1')
    if (c00 && c01) {
      pathThresholdRef.current = (c01.cx - c00.cx) * 0.50
    }
  }, [cellCenters])

  // partition constraints into region (fill) and path (line) types
  const { overlays, pathLines } = useMemo(() => {
    const size = board.length
    const result: CellOverlay[][] = Array(size).fill(null).map(() =>
      Array(size).fill(null).map(() => ({
        color: null,
        isSelected: false,
        isPreview: false,
        pathIndex: -1,
        isPathStart: false,
        isPathEnd: false,
        isPathCell: false,
      }))
    )
    const lines: PathLine[] = []

    // separate region and path constraints so they don't overwrite each other
    const regionConstraints = constraints.filter(c => getCategoryForType(c.constraintType) !== 'path')
    const pathConstraints = constraints.filter(c => getCategoryForType(c.constraintType) === 'path')

    // 1) region fills — active one on top
    const sortedRegion = [...regionConstraints].sort((a, b) => {
      if (a.id === activeConstraintId) return 1
      if (b.id === activeConstraintId) return -1
      return 0
    })
    for (const c of sortedRegion) {
      const isActive = c.id === activeConstraintId
      for (let i = 0; i < c.cells.length; i++) {
        const [r, cCol] = c.cells[i]
        const existing = result[r][cCol]
        result[r][cCol] = {
          ...existing,
          color: c.color,
          isSelected: existing.isSelected || isActive,
          isPathCell: false,
        }
      }
    }

    // 2) path markers — active one on top, preserve region fill
    const sortedPath = [...pathConstraints].sort((a, b) => {
      if (a.id === activeConstraintId) return 1
      if (b.id === activeConstraintId) return -1
      return 0
    })
    for (const c of sortedPath) {
      const isActive = c.id === activeConstraintId
      lines.push({ cells: c.cells, color: c.color, isActive, isPreview: false })
      for (let i = 0; i < c.cells.length; i++) {
        const [r, cCol] = c.cells[i]
        const existing = result[r][cCol]
        result[r][cCol] = {
          ...existing,
          isSelected: existing.isSelected || isActive,
          pathIndex: i,
          isPathStart: i === 0,
          isPathEnd: i === c.cells.length - 1,
          isPathCell: true,
        }
      }
    }

    // 3) currentCells preview
    if (currentCells.length > 0) {
      if (selectionMode === 'path') {
        lines.push({ cells: currentCells, color: previewColor, isActive: false, isPreview: true })
        for (let idx = 0; idx < currentCells.length; idx++) {
          const [r, c] = currentCells[idx]
          const existing = result[r][c]
          result[r][c] = {
            ...existing,
            isSelected: existing.isSelected,
            isPreview: true,
            pathIndex: idx,
            isPathStart: idx === 0,
            isPathEnd: idx === currentCells.length - 1 && idx > 0,
            isPathCell: true,
          }
        }
      } else {
        for (const [r, c] of currentCells) {
          const existing = result[r][c]
          result[r][c] = {
            ...existing,
            color: previewColor,
            isSelected: false,
            isPreview: true,
            isPathCell: false,
          }
        }
      }
    }

    // active lines on top
    lines.sort((a, b) => {
      if (a.isActive) return 1
      if (b.isActive) return -1
      return 0
    })

    return { overlays: result, pathLines: lines }
  }, [board.length, constraints, activeConstraintId, currentCells, selectionMode])

  const isSelecting = selectionMode !== 'none'

  // shared distance check for mouseenter / mousemove in path mode
  const tryPropagatePath = useCallback((r: number, c: number, e: React.MouseEvent) => {
    if (selectionMode !== 'path') {
      onCellMouseEnter?.(r, c)
      return
    }
    const gridRect = gridRef.current?.getBoundingClientRect()
    const center = cellCenters.get(`${r},${c}`)
    if (gridRect && center) {
      const mx = e.clientX - gridRect.left
      const my = e.clientY - gridRect.top
      const dist = Math.sqrt((mx - center.cx) ** 2 + (my - center.cy) ** 2)
      if (dist > pathThresholdRef.current) return
    }
    onCellMouseEnter?.(r, c)
  }, [selectionMode, cellCenters, onCellMouseEnter])

  return (
    <div
      ref={gridRef}
      className={`sudoku-grid${isSelecting ? ' selecting' : ''}`}
      role="grid"
      aria-label="数独棋盘"
      onMouseUp={onCellMouseUp}
      onMouseLeave={onCellMouseUp}
    >
      {board.map((row, i) => (
        <div key={i} className="sudoku-row">
          {row.map((val, j) => {
            const overlay = overlays[i]?.[j] ?? {
              color: null, isSelected: false, isPreview: false,
              pathIndex: -1, isPathStart: false, isPathEnd: false, isPathCell: false,
            }
            return (
              <div
                key={j}
                data-row={i}
                data-col={j}
                className={`sudoku-cell-wrap ${((i + 1) % blockRows === 0 && i < board.length - 1) ? 'border-bottom' : ''} ${((j + 1) % blockCols === 0 && j < board.length - 1) ? 'border-right' : ''}`}
              >
                <SudokuCell
                  value={val}
                  onChange={(v) => handleCellChange(i, j, v)}
                  readOnly={gridReadOnly}
                  maxValue={board.length}
                  overlay={overlay}
                  isSolved={solvedMask?.[i]?.[j] ?? false}
                  onMouseDown={() => onCellMouseDown?.(i, j)}
                  onMouseEnter={(e) => tryPropagatePath(i, j, e)}
                  onMouseMove={(e) => tryPropagatePath(i, j, e)}
                />
              </div>
            )
          })}
        </div>
      ))}

      <svg className="path-overlay-svg">
        {/* path lines */}
        {pathLines.map((line, li) => {
          const points = line.cells
            .map(([r, c]) => cellCenters.get(`${r},${c}`))
            .filter(Boolean) as { cx: number; cy: number }[]
          if (points.length < 2) return null
          return (
            <g key={li}>
              <polyline
                points={points.map(p => `${p.cx},${p.cy}`).join(' ')}
                fill="none"
                stroke={line.color}
                strokeWidth={line.isActive ? 4 : 3}
                strokeLinecap="round"
                strokeLinejoin="round"
                opacity={line.isPreview ? 0.7 : 0.85}
              />
              {[points[0], points[points.length - 1]].map((p, i) => (
                <circle
                  key={i}
                  cx={p.cx}
                  cy={p.cy}
                  r={i === 0 ? 5 : 4}
                  fill={line.color}
                  stroke={line.isActive ? '#fff' : 'rgba(255,255,255,0.5)'}
                  strokeWidth={1.5}
                />
              ))}
            </g>
          )
        })}

        {/* diagonal lines */}
        {(() => {
          const d = constraints.find(c => c.constraintType === 'diagonals')
          if (!d) return null
          const { w, h } = gridSizeRef.current
          const n = board.length
          const ptsMain = [
            { cx: 0, cy: 0 },
            ...Array.from({ length: n }, (_, i) => cellCenters.get(`${i},${i}`)).filter(Boolean) as { cx: number; cy: number }[],
            { cx: w, cy: h },
          ]
          const ptsAnti = [
            { cx: w, cy: 0 },
            ...Array.from({ length: n }, (_, i) => cellCenters.get(`${i},${n - 1 - i}`)).filter(Boolean) as { cx: number; cy: number }[],
            { cx: 0, cy: h },
          ]
          return (
            <g>
              <polyline
                points={ptsMain.map(p => `${p.cx},${p.cy}`).join(' ')}
                fill="none" stroke={d.color} strokeWidth={1.5}
                strokeLinecap="round" opacity={0.6}
              />
              <polyline
                points={ptsAnti.map(p => `${p.cx},${p.cy}`).join(' ')}
                fill="none" stroke={d.color} strokeWidth={1.5}
                strokeLinecap="round" opacity={0.6}
              />
            </g>
          )
        })()}
      </svg>

    </div>
  )
}

export { emptyBoard }
