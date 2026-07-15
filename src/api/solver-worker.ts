/// <reference types="vite/client" />

import { initSync, solve, validate } from "../../pkg/solver_wasm";

// ── WASM initialisation ──────────────────────────────────────

let ready = false;
const initPromise: Promise<void> = (async () => {
  const wasmUrl = new URL("../../pkg/solver_wasm_bg.wasm", import.meta.url);
  const resp = await fetch(wasmUrl);
  if (!resp.ok) throw new Error(`Failed to load WASM: ${resp.status}`);
  const bytes = await resp.arrayBuffer();
  initSync(bytes);
  ready = true;
})();

// ── Message types ────────────────────────────────────────────

interface WorkerRequest {
  id: number;
  type: "solve" | "validate";
  payload: {
    board: number[][];
    params: Record<string, unknown>;
  };
}

interface WorkerResponse {
  id: number;
  type: "solve" | "validate";
  result: unknown;
  error?: never;
}

interface WorkerError {
  id: number;
  type: "solve" | "validate";
  result?: never;
  error: string;
}

// ── Handler ──────────────────────────────────────────────────

self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  const { id, type, payload } = e.data;

  try {
    await initPromise;
    if (!ready) throw new Error("WASM initialisation failed");

    const jsonIn = JSON.stringify(payload);
    const jsonOut = type === "solve" ? solve(jsonIn) : validate(jsonIn);
    const result = JSON.parse(jsonOut);

    const resp: WorkerResponse = { id, type, result };
    self.postMessage(resp);
  } catch (err) {
    const resp: WorkerError = {
      id,
      type,
      error: (err as Error).message || "Unknown worker error",
    };
    self.postMessage(resp);
  }
};
