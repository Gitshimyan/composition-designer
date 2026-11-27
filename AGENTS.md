# AGENTS.md — Composition Designer 工作规则

本文件告诉 AI 如何在本仓库工作。**文档不是强制边界**；真正的阻断由 `cargo` 门禁、
Schema 校验与变更检查承担。门禁建立之前，修改范围由任务卡的“授权”与
`changes/` 声明自控，**不要声称执行过尚未存在的验证**。

## 0. 开始任何行动前

1. 读 [`working/plan.md`](./working/plan.md)，只取其中一张任务卡。
2. 读设计基线 [`docs/design/v0.1-design.md`](./docs/design/v0.1-design.md)
   的相关章节（执行顺序见 §16，阶段见 §11），以及 [`docs/spec/`](./docs/spec/README.md)
   中与目标对象相关的冻结规范。
3. 读 [`docs/ai/architecture-entry.md`](./docs/ai/architecture-entry.md) 与目标目录的 README，
   确认当前真实状态，不要沿用记忆里的字段或命令。
4. 确认任务卡声明的验收条件现在是否可执行；不可执行就缩小卡的范围，而不是伪造结果。

## 1. 工作模式（默认 Modify）

| 模式 | 允许 | 默认 |
|---|---|---|
| Inspect | 只读设计基线、规范、声明、计划与影响分析 | 允许 |
| Modify | 目标 crate 的 `src/`、测试、crate 内文档；任务卡指定的 `docs/` 规范与计划 | 普通实现任务默认允许 |
| Compose | `examples/**` 的组合声明、`.designer/**` 表现数据、`docs/` 中的组合样例 | 需求涉及组合时允许 |
| Extend | 新 crate、新 TOML 字段、新命令/操作、新 Adapter、新示例类型 | **需要用户明确授权** |
| Rearchitect | 领域模型、对象关系、`schema_version`、命令协议、工程配置、设计基线 | **需要用户明确授权** |

- 设计基线文档的任何修改都按 Rearchitect 处理；
- 同一任务同时需要 Modify 和 Compose 时，Change Manifest 必须同时声明两个 operation；
- 拿不准时按更高一级授权处理，先报告阻塞。

## 2. 默认禁止

- 未经 Rearchitect 授权修改 `docs/design/v0.1-design.md`（设计基线）；
- 在 TOML 中嵌入脚本、命令或任意可执行逻辑；
- 让 GUI / Agent / CLI 各自维护项目模型；所有读写必须经 Composition Core；
- 让 GUI 或 Agent 直接写项目文件，绕过解析、验证与事务；
- 用“尽量兼容”、自动 Adapter 或静默修复处理不兼容；
- 把画布坐标、视口、面板状态写进 `composition/` 或任何项目语义文件；
- 把 Dependency 推断为 Binding，或把 Binding 推断为 Connection；
- 在业务模块/UI 中加入旧格式判断、静默 fallback 或“以后可能需要”的兼容分支；
- 为通过检查在下游增加防御性兼容代码；
- 修改非目标 crate / 非目标目录。

需要边界变化时，**报告阻塞**并给出最小扩展方案，不要偷偷越界。

## 3. 变更声明与校验

1. 从 [`changes/000-template.yaml`](./changes/000-template.yaml) 复制一份 Change Manifest；
2. 填写 `intent`、`authorization`、`targets`、`operations`、`scope`、`impact`、`validation`；
3. 实现前确认影响范围（M2 起用命令生成计划 / 影响分析）；
4. 结卡时逐项核对实际改动文件与声明范围。

### 3.1 远端推送规则

默认只在本地 `git commit` 管理历史。**只有用户明确说明“推送到远端/发布到 GitHub”时才执行
`git push`**；阶段评审需要推送时，先确认目标仓库与分支。不得在普通实现、修文档或结卡后
自动推送。

当前阶段（v0.1 M0–M5 完成、v0.2 计划中）**没有自动变更校验**；声明是书面契约，结卡时必须人工核对。
仓库门禁建立后，越权或声明不一致必须视为失败，不能绕过。

## 4. 实现流程（M1 起）

1. 在目标 crate 读 `docs/spec/` 与类型定义，不猜字段与命令；
2. 只使用 Core 暴露的 Command / Query；UI 不重复实现契约规则；
3. 新行为先写失败用例，再实现（M1 起 `cargo test`）；
4. 模块边界与依赖方向遵循 [`crates/README.md`](./crates/README.md)；
5. 结卡前运行任务卡列出的验收命令；没有命令可跑时，明确写“未执行”。

## 5. 新增 crate / Schema 字段 / 命令（需授权）

- 新 crate：在 `crates/<name>` 建立，注册到 Cargo workspace，并在
  [`docs/ai/architecture-entry.md`](./docs/ai/architecture-entry.md) 补职责与依赖方向；
- 新 TOML 字段：先改 `docs/spec/composition-schema-v1.md` 与错误码，再改校验、样例与 Core，顺序不能反；
- 新命令：先定义输入 Schema、前置条件、结果结构与错误码，再实现；
- 新 Adapter：MCP 等只是协议适配器，不得改变 Core 语义；
- 新示例项目：放 `examples/`，必须带 README 与预期诊断。

## 6. 完成定义

- 任务卡清单勾完，验收条件全部满足（不是“大部分满足”）；
- 变更范围与 Change Manifest 一致；
- 受影响的规范、样例与测试同步更新；
- 回写 `working/tasks/<card>.md` 与 `working/plan.md` 的状态、结果、下一步；
- 不把“没有失败”当成“验证通过”。

## 7. 常用命令

```powershell
# M1（已完成，005–008）
cargo test --workspace
cargo test -p composition-cli
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p composition-cli -- validate examples/minimal-project --json

# M2（009–011 完成）
cargo test -p composition-editor
cargo run -p composition-cli -- inspect examples/minimal-project

# M3（012–015 完成）
cd apps/designer
npm run setup
npm run build
npm run desktop
npm start

# M4（016–018 完成）
cargo run -p composition-cli -- query examples/minimal-project validate
cargo run -p composition-cli -- jsonl examples/minimal-project
cargo run -p composition-mcp -- --project examples/minimal-project
cargo test -p composition-mcp --test e2e

# M5（019 完成）
cargo test --workspace
cargo test --manifest-path apps/designer/src-tauri/Cargo.toml
cargo test -p composition-mcp --test e2e
# v0.1 验收：docs/acceptance-v0.1.md
```

构建与测试通过宿主机任务（host_task）执行；分析、编辑在沙箱内完成。
`ai.cargo-test` / `ai.cargo-clippy` / `ai.cargo-fmt` 已批准可用；新增命令需再登记。

## 8. 归档与例外

- 过期或否定的方案进 `docs/adr/`，不留在代码注释里“以后再说”；
- 试点、旧 Schema、临时样例放 `working/tmp/` 或仓库外归档目录，不恢复旧声明文件；
- 任何例外必须在任务卡与 Change Manifest 中写明范围、责任与移除条件。
