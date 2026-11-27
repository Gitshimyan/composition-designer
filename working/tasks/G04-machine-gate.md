# G04 — 机器授权/覆盖门禁

- 状态：done
- coverage_status：complete
- open_items：无
- 目标：让 Manifest、实际 diff 与需求覆盖状态可自动核对，先报告、后阻断；避免“声明覆盖”继续只靠人工。
- authorization：
  - required：extend, rearchitect
  - approval：
    - extend：granted（用户 2026-11-27 明确授权）
    - rearchitect：granted（用户 2026-11-27 明确授权）
- 来源：`docs/reviews/2026-11-27-authorization-audit.md` §9.1
- 变更声明：`changes/G04-machine-gate.yaml`（已创建）

## 范围

1. 实现一个仓库内检查工具（脚本或 crate）：
   - 读取 `changes/*.yaml` 与 `git diff`；
   - 校验每个变更文件落在 `targets` 内；
   - 校验实际操作有对应 `operations` / `scope.allowed`；
   - 校验敏感路径（`docs/design/**`、`docs/spec/**`、`AGENTS.md`、workspace 配置）
     需要对应 Rearchitect 授权声明；
   - 校验 Extend 项（新 crate/字段/命令/Adapter/示例类型）有声明；
   - 校验 Compose 项（`examples/**`、`working/pilots/**`）有声明；
   - 校验 `unresolved_items` 存在且 `coverage_status` 与任务卡一致；
   - 输出未声明路径、授权不足、被 forbidden 命中的文件。
2. 先以报告模式接入 CI；收集误报、补齐规则后再切为阻断模式。
3. 提供测试夹具：通过、越权、缺少授权、敏感路径未声明、coverage 不一致。

## 清单

- [x] 实现 `composition-change-gate`（Manifest × diff × authorization/scope/coverage）
- [x] 支持 report / block 两种模式
- [x] 提供通过、越权、缺少授权、敏感路径、coverage 不一致测试
- [x] CI 接入 report 模式（`fetch-depth: 0`）
- [x] 在现有仓库上运行 report，并验证 block 模式退出码
- [x] 更新 AGENTS、模板、README、architecture-entry
- [x] 创建 `changes/G04-machine-gate.yaml`

## 非目标

- 不自动授予权限；
- 不替代人工评审；
- 不在工具中执行项目语义校验（仍由 Core 负责）；
- 不把“Manifest 已声明”等同于 Core 已接受变更。

## 验收

- 工具能稳定比较 `targets` 与实际 diff，并在测试夹具上给出预期结果；
- 报告模式在现有仓库上运行，输出可解释的未声明/授权不足项；
- 阻断模式启用前有明确文档、误报处理和绕过审计；
- CI 中失败信息可定位到文件、Manifest 与所需授权；
- 不修改领域模型、TOML Schema 或产品功能。

## 结果 / 遗留问题

- 结果：
  - 新增 [`tools/change-gate`](../../tools/change-gate/README.md)：`composition-change-gate`；
  - 检查 targets、授权/批准、operation、敏感路径、Compose、Extend、forbidden、
    `coverage_status` 与 `unresolved_items`；
  - 7 个单元测试 + 1 个真实仓库 report 集成测试；
  - CI 已运行 report 模式；block 模式已验证在 coverage 不一致时退出 1；
  - 集成 PR #2 CI 通过（run 45997141339）并已合并；合并后 main push CI 通过
    （run 45997795133）；
  - 失败探针 PR #1 CI fail 且 `mergeStateStatus=BLOCKED`，已关闭；分支保护有效；
  - `changes/000-template.yaml` 增加 `coverage_status`；
  - **UW-010 关闭**（机器门禁 report 实现）。
- 未启用 / 后续：
  - UW-030 已由 G05 关闭：CI 切换为 block 模式；
  - block 模式仍只检查声明与 diff，不替代 Core/人工评审。
- 验证：
  - `cargo test --workspace`：163 passed；
  - `cargo clippy --workspace --all-targets -- -D warnings`：通过；
  - `cargo fmt --all --check`：通过；
  - `npm run build`：通过（未改 UI）；
  - 真实仓库 report：0 error（G04 结卡后）；block 模式在已知 mismatch 上退出 1。
- 下一步：M6 收口完成后可选择启用 block 模式；G04 不自动授予权限。
