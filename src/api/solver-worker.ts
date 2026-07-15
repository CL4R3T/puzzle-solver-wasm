/// <reference types="vite/client" />

// wasm-pack --target bundler initialises at import time (no explicit init())
import { solve, validate } from "../../pkg/solver_wasm";

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
