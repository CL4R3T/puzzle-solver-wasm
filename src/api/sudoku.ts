type Board = number[][];
type BoxShape = [number, number];

export interface SolveResult {
  success: boolean;
  solution: Board | null;
  message: string;
  solve_time_ms: number | null;
  steps: Record<string, unknown>[] | null;
}

export interface ValidateResult {
  valid: boolean;
  unique_solution: boolean | null;
  message: string;
}

export interface SolveParams {
  boxShape?: BoxShape;
  diagonals?: boolean;
  cages?: { cells: [number, number][]; sum: number }[];
  thermos?: [number, number][][];
  palindromes?: [number, number][][];
}

// ── Worker setup ─────────────────────────────────────────────

const WORKER_TIMEOUT_MS = 60_000;

let _worker: Worker | null = null;
let _nextId = 1;
const _pending = new Map<
  number,
  {
    resolve: (v: unknown) => void;
    reject: (e: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  }
>();

function getWorker(): Worker {
  if (!_worker) {
    _worker = new Worker(new URL("./solver-worker.ts", import.meta.url), {
      type: "module",
    });
    _worker.onmessage = (e: MessageEvent) => {
      const { id, result, error } = e.data;
      const handler = _pending.get(id);
      if (!handler) return;
      _pending.delete(id);
      clearTimeout(handler.timer);
      if (error) {
        handler.reject(new Error(error as string));
      } else {
        handler.resolve(result);
      }
    };
    _worker.onerror = (ev: ErrorEvent) => {
      // Worker-level error; reject all pending
      for (const [id, h] of _pending) {
        clearTimeout(h.timer);
        h.reject(new Error(ev.message || "Worker error"));
        _pending.delete(id);
      }
    };
  }
  return _worker;
}

function postToWorker(
  type: "solve" | "validate",
  payload: { board: Board; params: Record<string, unknown> },
): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const id = _nextId++;
    const timer = setTimeout(() => {
      _pending.delete(id);
      reject(new Error("求解超时，请尝试简化题目"));
    }, WORKER_TIMEOUT_MS);
    _pending.set(id, { resolve, reject, timer });
    getWorker().postMessage({ id, type, payload });
  });
}

// ── Public API (same signatures, different implementation) ────

function buildParams(params: SolveParams): Record<string, unknown> {
  const p: Record<string, unknown> = {};
  if (params.boxShape) p.box_shape = params.boxShape;
  if (params.diagonals) p.diagonals = true;
  if (params.cages && params.cages.length > 0) p.cages = params.cages;
  if (params.thermos && params.thermos.length > 0) p.thermos = params.thermos;
  if (params.palindromes && params.palindromes.length > 0)
    p.palindromes = params.palindromes;
  return p;
}

export async function solveSudoku(
  board: Board,
  params: SolveParams = {},
): Promise<SolveResult> {
  const payload = { board, params: buildParams(params) };
  const result = await postToWorker("solve", payload);
  return result as SolveResult;
}

export async function validateSudoku(
  board: Board,
  params: SolveParams = {},
): Promise<ValidateResult> {
  const payload = { board, params: buildParams(params) };
  const result = await postToWorker("validate", payload);
  return result as ValidateResult;
}
