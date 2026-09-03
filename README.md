# 纸笔谜题求解器

一个运行在浏览器中的数独及变体求解器。核心算法由 Rust 编写并编译为 WebAssembly，计算任务在 Web Worker 中执行，不依赖后端服务，也不会阻塞页面交互。

目前项目提供数独求解界面，并支持杀手数独、对角线、温度计、慢温度计和回文线等附加约束。

## 功能

- 校验当前盘面是否违反规则
- 求解标准数独及多种数独变体
- 支持 4×4 到 16×16 的合数阶盘面
- 支持自定义宫格形状，例如 6×6 盘面的 2×3 宫
- 在棋盘上拖拽绘制区域约束和路径约束
- 所有求解均在浏览器本地完成

### 支持的约束

| 约束 | 规则 | 创建方式 |
| --- | --- | --- |
| 行、列、宫 | 同一区域内的数字不重复 | 默认启用 |
| 对角线 | 两条主对角线上的数字分别不重复 | 点击工具栏开关 |
| 杀手笼 | 笼内数字不重复，且总和等于目标值 | 拖拽选择相邻区域，再设置目标和 |
| 温度计 | 从圆头到末端严格递增 | 按顺序拖拽绘制路径 |
| 慢温度计 | 从起点到末端单调不减 | 按顺序拖拽绘制路径 |
| 回文线 | 关于路径中心对称的格子取值相同 | 按顺序拖拽绘制路径 |

## 本地运行

### 环境要求

- Node.js 与 npm
- Rust stable 工具链
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)

安装依赖并启动开发服务器：

```bash
npm install
npm run dev
```

打开终端输出中的本地地址（Vite 默认使用 `http://localhost:5173`）。`npm run dev` 会先编译 WASM、检查 TypeScript，再启动开发服务器。

创建并预览生产构建：

```bash
npm run build
npm run preview
```

构建结果位于 `dist/`。由 `wasm-pack` 生成的中间文件位于 `pkg/`，不应手动修改。

## 使用方法

1. 在棋盘中填写已知数字；空格保持为空。
2. 如需更改盘面大小或宫格形状，点击“设置”。
3. 如需添加变体规则，从工具栏选择区域、路径或对角线约束：
   - 区域约束使用四方向相邻的连续格子；
   - 路径约束可沿横、竖或斜向的相邻格子绘制；
   - 创建后可在约束侧栏中修改类型、参数或颜色。
4. 点击“校验”检查当前输入，或点击“求解”查看完整答案。

“清空”会同时移除盘面数字和已创建的附加约束。

## 工作原理

```text
React 界面
    │
    ▼
TypeScript 异步 API
    │
    ▼
Web Worker
    │
    ▼
WASM 绑定层（solver-wasm）
    │
    ▼
Rust 求解引擎（solver-core）
```

`solver-core` 将题目建模为约束满足问题（CSP），使用位掩码保存候选数，通过约束传播和可组合推理策略缩小搜索空间，并在需要时结合 MRV 启发式进行回溯搜索。前端仅负责输入、约束编辑和结果展示；Rust 核心不依赖浏览器，可独立测试。

## 项目结构

```text
crates/
├── solver-core/       Rust 求解引擎、约束和推理策略
└── solver-wasm/       wasm-bindgen 对外接口
src/
├── api/               Worker 通信与前端求解 API
├── components/        React 页面和棋盘组件
└── constraints/       前端约束定义
.github/workflows/     持续集成与 GitHub Pages 部署
```

## 常用命令

```bash
# 运行 Rust 核心测试
cargo test -p solver-core

# 检查 Rust 格式
cargo fmt --all --check

# 运行 Rust 静态检查
cargo clippy --all-targets -- -D warnings

# 单独重新生成 WASM 包
npm run build:wasm

# 编译 WASM、检查 TypeScript 并打包前端
npm run build
```

## CI 与部署

每次 push 和 pull request 都会运行 Rust 格式检查、Clippy、核心测试以及完整前端构建。推送到 `main` 分支后，构建产物会自动部署到 GitHub Pages。

## 技术栈

- Rust、`wasm-bindgen`、WebAssembly
- React、TypeScript
- Web Worker
- Vite
- GitHub Actions、GitHub Pages

## License

[MIT](LICENSE)
