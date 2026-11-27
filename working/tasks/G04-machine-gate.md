# G04 — 机器授权/覆盖门禁

- 状态：deferred（026 之后执行）
- 目标：让 Manifest、实际 diff 与需求覆盖状态可自动核对，先报告、后阻断；避免“声明覆盖”继续只靠人工。
- 授权：Extend + Rearchitect（新增门禁工具与工程质量策略）
- 来源：`docs/reviews/2026-11-27-authorization-audit.md` §9.1
- 变更声明：`changes/G04-machine-gate.yaml`（待开始后创建）

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

- 尚未开始；依赖 G01–G03 的规则与登记册稳定后再实现。
