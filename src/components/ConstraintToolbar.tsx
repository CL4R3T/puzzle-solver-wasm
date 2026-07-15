import type { SelectionMode } from './SudokuGrid'

interface ConstraintToolbarProps {
  selectionMode: SelectionMode
  onSelectMode: (mode: SelectionMode) => void
  diagonalsEnabled: boolean
  onToggleDiagonals: () => void
  constraintCount: number
  onToggleSidebar: () => void
  disabled?: boolean
}

export function ConstraintToolbar({
  selectionMode,
  onSelectMode,
  diagonalsEnabled,
  onToggleDiagonals,
  constraintCount,
  onToggleSidebar,
  disabled = false,
}: ConstraintToolbarProps) {
  return (
    <div className="constraint-toolbar">
      <button
        type="button"
        className={selectionMode === 'none' ? 'active' : ''}
        onClick={() => onSelectMode('none')}
        disabled={disabled}
      >
        指针
      </button>
      <button
        type="button"
        className={selectionMode === 'region' ? 'active' : ''}
        onClick={() => onSelectMode('region')}
        disabled={disabled}
      >
        区域选区
      </button>
      <button
        type="button"
        className={selectionMode === 'path' ? 'active' : ''}
        onClick={() => onSelectMode('path')}
        disabled={disabled}
      >
        路径选区
      </button>
      <span className="toolbar-sep">|</span>
      <button
        type="button"
        className={diagonalsEnabled ? 'active' : ''}
        onClick={onToggleDiagonals}
        disabled={disabled}
      >
        对角线
      </button>
      <span className="toolbar-sep">|</span>
      <button
        type="button"
        onClick={onToggleSidebar}
        disabled={disabled}
      >
        约束列表 {constraintCount > 0 ? `(${constraintCount})` : ''}
      </button>
    </div>
  )
}
