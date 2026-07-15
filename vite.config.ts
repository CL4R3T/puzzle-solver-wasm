import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [react(), wasm()],
  assetsInclude: ["**/*.wasm"],
  worker: {
    format: "es",
  },
  optimizeDeps: {
    exclude: ["pkg/solver_wasm"],
  },
});
