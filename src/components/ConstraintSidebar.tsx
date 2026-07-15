import type {
  ConstraintInstance,
  ConstraintDef,
} from "../constraints/definitions";
import { CONSTRAINT_DEFS, PALETTE } from "../constraints/definitions";

interface ConstraintSidebarProps {
  constraints: ConstraintInstance[];
  activeConstraintId: string | null;
  onSelect: (id: string | null) => void;
  onUpdate: (id: string, patch: Partial<ConstraintInstance>) => void;
  onDelete: (id: string) => void;
  onClose: () => void;
}

export function ConstraintSidebar({
  constraints,
  activeConstraintId,
  onSelect,
  onUpdate,
  onDelete,
  onClose,
}: ConstraintSidebarProps) {
  return (
    <div className="constraint-sidebar">
      <div className="constraint-sidebar-header">
        <span>约束列表</span>
        <button type="button" onClick={onClose} aria-label="关闭">
          ×
        </button>
      </div>

      <div className="constraint-list">
        {constraints.length === 0 ? (
          <div className="constraint-list-empty">
            暂无约束。使用上方工具条创建区域或路径选区。
          </div>
        ) : (
          constraints.map((c) => (
            <div
              key={c.id}
              className={`constraint-item${c.id === activeConstraintId ? " active" : ""}`}
              onClick={() =>
                onSelect(c.id === activeConstraintId ? null : c.id)
              }
            >
              <div
                className="constraint-item-color"
                style={{ background: c.color }}
              />
              <div className="constraint-item-info">
                <div className="constraint-item-name">
                  {CONSTRAINT_DEFS.find((d) => d.type === c.constraintType)
                    ?.label ?? c.constraintType}
                </div>
                <div className="constraint-item-summary">
                  {c.constraintType === "cages"
                    ? `sum=${c.params.sum ?? "?"}, ${c.cells.length}格`
                    : c.constraintType === "diagonals"
                      ? "主对角线+副对角线"
                      : `${c.cells.length}格`}
                </div>
              </div>
              <button
                type="button"
                className="constraint-item-delete"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(c.id);
                }}
                aria-label="删除约束"
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>

      {activeConstraintId && (
        <ConstraintEditor
          instance={constraints.find((c) => c.id === activeConstraintId)!}
          onChange={(patch) => onUpdate(activeConstraintId, patch)}
          onDelete={() => onDelete(activeConstraintId)}
        />
      )}
    </div>
  );
}

interface ConstraintEditorProps {
  instance: ConstraintInstance;
  onChange: (patch: Partial<ConstraintInstance>) => void;
  onDelete: () => void;
}

function ConstraintEditor({
  instance,
  onChange,
  onDelete,
}: ConstraintEditorProps) {
  const def = CONSTRAINT_DEFS.find((d) => d.type === instance.constraintType);
  if (!def) return null;

  return (
    <div className="constraint-editor">
      <h3>编辑：{def.label}</h3>

      <div className="editor-field">
        <label>约束类型</label>
        <select
          value={instance.constraintType}
          onChange={(e) => {
            const newType = e.target.value;
            const newDef = CONSTRAINT_DEFS.find((d) => d.type === newType);
            onChange({
              constraintType: newType,
              params: newDef ? defaultParams(newDef) : {},
            });
          }}
        >
          {CONSTRAINT_DEFS.filter((d) => d.category === def.category).map(
            (d) => (
              <option key={d.type} value={d.type}>
                {d.label}
              </option>
            ),
          )}
        </select>
      </div>

      <div className="editor-field">
        <label>颜色</label>
        <div className="color-palette">
          {PALETTE.map((color) => (
            <div
              key={color}
              className={`color-swatch${instance.color === color ? " selected" : ""}`}
              style={{ background: color }}
              onClick={() => onChange({ color })}
            />
          ))}
        </div>
      </div>

      {def.params.map((param) => (
        <div key={param.key} className="editor-field">
          <label>{param.label}</label>
          {param.type === "number" ? (
            <input
              type="number"
              min={param.min}
              max={param.max}
              value={String(
                instance.params[param.key] ?? param.defaultValue ?? "",
              )}
              onChange={(e) => {
                const v =
                  e.target.value === "" ? undefined : Number(e.target.value);
                onChange({ params: { ...instance.params, [param.key]: v } });
              }}
            />
          ) : param.type === "boolean" ? (
            <input
              type="checkbox"
              checked={Boolean(
                instance.params[param.key] ?? param.defaultValue,
              )}
              onChange={(e) =>
                onChange({
                  params: { ...instance.params, [param.key]: e.target.checked },
                })
              }
            />
          ) : (
            <select
              value={String(
                instance.params[param.key] ?? param.defaultValue ?? "",
              )}
              onChange={(e) =>
                onChange({
                  params: { ...instance.params, [param.key]: e.target.value },
                })
              }
            >
              {param.options?.map((opt) => (
                <option key={String(opt.value)} value={String(opt.value)}>
                  {opt.label}
                </option>
              ))}
            </select>
          )}
        </div>
      ))}

      <div className="editor-actions">
        <button type="button" className="danger" onClick={onDelete}>
          删除
        </button>
      </div>
    </div>
  );
}

function defaultParams(def: ConstraintDef): Record<string, unknown> {
  const p: Record<string, unknown> = {};
  for (const param of def.params) {
    p[param.key] = param.defaultValue;
  }
  return p;
}
