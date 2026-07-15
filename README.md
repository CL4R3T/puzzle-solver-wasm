# 谜题求解器（WASM 版）

基于 Rust + WebAssembly 的纯前端谜题求解器，支持数独及其变体（杀手数独、对角线数独、温度计、回文线等）。所有运算在浏览器本地完成，无需后端服务器。

## 快速开始

```bash
npm install
npm run dev       # 启动 Vite 开发服务器，访问 http://localhost:5173
```

生产构建：

```bash
npm run build     # 编译 WASM + TypeScript + Vite 打包
npm run preview   # 预览生产构建
```

## 技术栈

| 层           | 技术                          |
| ------------ | ----------------------------- |
| 求解引擎     | Rust（约束传播 + 回溯搜索）   |
| 浏览器运行时 | WebAssembly（wasm-pack 编译） |
| UI 框架      | React 18 + TypeScript 5       |
| 构建工具     | Vite 5                        |
| 并发         | Web Worker（求解不阻塞 UI）   |
| CI/CD        | GitHub Actions → GitHub Pages |

## 项目结构

```
puzzle-solver-wasm/
├── crates/
│   ├── solver-core/        # 纯 Rust 求解引擎（平台无关，可 cargo test）
│   │   └── src/
│   │       ├── solver.rs        # 核心求解器：约束传播 + 回溯 + MRV 启发式
│   │       ├── constraint.rs    # Constraint trait + propagate_units 共享逻辑
│   │       ├── constraints/     # 7 种约束实现
│   │       │   ├── row.rs            # 行不重复
│   │       │   ├── column.rs         # 列不重复
│   │       │   ├── box_constraint.rs # 宫格不重复（可自定义形状）
│   │       │   ├── diagonal.rs       # 对角线不重复（X-Sudoku）
│   │       │   ├── killer_cage.rs    # 杀手笼（求和 + 不重复）
│   │       │   ├── thermo.rs         # 温度计（严格递增路径）
│   │       │   └── palindrome.rs     # 回文线（对称路径）
│   │       ├── state.rs         # 位掩码棋盘状态
│   │       ├── types.rs         # SolveResult / ValidationResult
│   │       └── params.rs        # JSON 参数解析
│   │
│   └── solver-wasm/         # WASM 绑定层（wasm-bindgen）
│       └── src/lib.rs            # solve() / validate() 入口
│
├── pkg/                     # wasm-pack 构建输出（gitignored）
├── src/                     # 前端
│   ├── api/
│   │   ├── sudoku.ts            # 公共 API（solveSudoku / validateSudoku）
│   │   └── solver-worker.ts     # Web Worker 加载 WASM
│   ├── components/              # React UI 组件
│   └── constraints/             # 约束 UI 定义
│
├── .github/workflows/ci.yml    # GitHub Actions CI/CD
└── vite.config.ts
```

## 架构

```
用户操作 UI
    │
    ▼
React 组件 (Sudoku.tsx)
    │
    ▼
api/sudoku.ts        ← 异步接口（solveSudoku / validateSudoku）
    │
    ├─ 开发模式：直接 import WASM（Vite 热更新）
    └─ 生产模式：Web Worker 加载 .wasm
    │
    ▼
solver-worker.ts     ← WASM 胶水层（JSON ↔ Rust 字符串）
    │
    ▼
solver-wasm crate    ← #[wasm_bindgen] 入口函数
    │
    ▼
solver-core crate    ← 约束传播 + 回溯搜索
```

求解算法：**位掩码 CSP → 固定点约束传播 → 回溯搜索（MRV 启发式）**，和 Python 版算法完全一致。

## 支持的约束类型

| 约束                 | 前端交互     | 说明                       |
| -------------------- | ------------ | -------------------------- |
| 行/列/宫格           | 自动（内置） | 标准数独规则               |
| 对角线               | 布尔开关     | 主/副对角线数字不重复      |
| 杀手笼 (Killer Cage) | 区域拖拽     | 笼内不重复，求和等于目标值 |
| 温度计 (Thermometer) | 路径绘制     | 沿路径值严格递增           |
| 回文线 (Palindrome)  | 路径绘制     | 路径两端对称相等           |

## 棋盘配置

- 支持 4×4 到 16×16（非质数边长）
- 可自定义宫格形状（如 6×6 用 2×3 宫格）
- 前端 UI 提供宫格形状设置对话框

## 运行测试

```bash
# Rust 求解器单元测试（37 个）
cargo test -p solver-core

# WASM 集成测试
wasm-pack test --node crates/solver-wasm

# 前端类型检查
npx tsc --noEmit
```

## CI / 部署

推送 `main` 分支自动触发：

1. `cargo fmt` + `clippy` + `cargo test`（Rust 静态检查 + 测试）
2. `wasm-pack build` + `wasm-pack test`（WASM 编译 + 测试）
3. `tsc` + `vite build`（前端类型检查 + 打包）
4. 部署到 GitHub Pages

PR 不部署，只运行前三步。

## 与 Python 后端的关系

原 `puzzle-solver-backend/`（FastAPI）和 `puzzle-solver-frontend/`（React）完整保留，不做修改。

本项目的求解器是用 Rust 重写的相同算法，通过 WASM 在浏览器中本地运行。相同的 JSON 输入产生相同的输出，但：

- **不需要服务器** — 静态文件托管即可（GitHub Pages 免费）
- **离线可用** — 所有运算在浏览器本地完成
- **性能更高** — 位掩码运算是原生 CPU 指令，无 Python deepcopy 开销
- **隐私** — 用户数据不离开浏览器

## License

MIT
