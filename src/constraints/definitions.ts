export type ConstraintCategory = "region" | "path" | "toggle";

export interface ConstraintParamDef {
  key: string;
  label: string;
  type: "number" | "boolean" | "select";
  required: boolean;
  defaultValue?: unknown;
  options?: { label: string; value: unknown }[];
  min?: number;
  max?: number;
}

export interface ConstraintDef {
  type: string;
  label: string;
  category: ConstraintCategory;
  params: ConstraintParamDef[];
  defaultColor: string;
}

export interface ConstraintInstance {
  id: string;
  constraintType: string;
  cells: [number, number][];
  params: Record<string, unknown>;
  color: string;
}

export const CONSTRAINT_DEFS: ConstraintDef[] = [
  {
    type: "cages",
    label: "杀手笼",
    category: "region",
    params: [
      {
        key: "sum",
        label: "目标和",
        type: "number",
        required: true,
        min: 1,
        max: 405,
      },
    ],
    defaultColor: "#f87171",
  },
  {
    type: "thermometer",
    label: "温度计",
    category: "path",
    params: [],
    defaultColor: "#fb923c",
  },
  {
    type: "palindrome",
    label: "回文线",
    category: "path",
    params: [],
    defaultColor: "#a78bfa",
  },
  {
    type: "diagonals",
    label: "对角线",
    category: "toggle",
    params: [],
    defaultColor: "#60a5fa",
  },
];

export const PALETTE = [
  "#f87171",
  "#fb923c",
  "#fbbf24",
  "#34d399",
  "#60a5fa",
  "#a78bfa",
  "#e879f9",
  "#f472b6",
];

export function getNextUnusedColor(used: ConstraintInstance[]): string {
  const usedColors = new Set(used.map((c) => c.color));
  for (const color of PALETTE) {
    if (!usedColors.has(color)) return color;
  }
  return PALETTE[used.length % PALETTE.length];
}

export function getCategoryForType(constraintType: string): ConstraintCategory {
  const def = CONSTRAINT_DEFS.find((d) => d.type === constraintType);
  return def?.category ?? "region";
}

export function getDefaultTypeForCategory(
  category: ConstraintCategory,
): string {
  const def = CONSTRAINT_DEFS.find((d) => d.category === category);
  return def?.type ?? "cages";
}
