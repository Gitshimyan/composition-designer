# 005 — M1 Rust 工程与领域类型

- 状态：done
- 目标：建立 Cargo workspace 与 `composition-domain` crate，把 M0 冻结的 ID、契约、诊断与
  Revision 类型落成可编译、可测试的 Rust 领域模型；不含 TOML 解析（006）。
- 授权：Extend（新 crate、workspace 工程配置）
- 变更声明：`changes/005-rust-domain.yaml`

> 授权审计（2026-11-27）：审计建议 **Extend；首次建立全局 workspace 基线部分另需 Rearchitect**（条件性）；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

- coverage_status：complete
- open_items：无
- 历史回溯：[2026-11-27 未实现工作回溯](../../docs/reviews/2026-11-27-unimplemented-work-backfill.md)

## 背景

M0 已冻结规范与样例，但仓库没有任何 Rust 代码。M1 需要一个所有后续 crate 共享的领域层：
ID/版本的类型化解析、契约结构、组合对象、诊断与 Revision。先把这些类型和自洽规则做对，
006 才能用 `toml_edit` 加载声明而不把 TOML 语法泄漏进领域模型。

crate 拆分决策：M1 先只建立 `composition-domain`；`composition-toml` / `resolver` /
`validator` / `editor` / `protocol` / `app` 在对应任务中按依赖方向加入，避免空 crate 和过早边界。

## 清单

- [x] 建立根 `Cargo.toml` workspace（resolver 2、edition 2021）与 `crates/composition-domain`
- [x] `ids`：`ModuleId` / `TypeId` / `LocalId` / `Version` 的解析、校验、Display、Ord
- [x] `schema`：`SchemaVersion`（v0.1 只支持 1）
- [x] `value`：`ConfigValue`（string / integer / float / boolean / array；无 object）
- [x] `contract`：Property / Slot / Port / Dependency 契约与约束自洽检查
- [x] `module`：`ModuleDefinition` 与 primitive/composite 规则
- [x] `composition`：Instance / Binding / Connection / PortRef / Composition
- [x] `diagnostic`：`ErrorCode` 全覆盖、Severity、`ObjectId`、`Diagnostic` 与稳定排序
- [x] `revision`：`SourceDigest` 格式与 `Revision`
- [x] `model`：`Project`、`DependencyResolution`、`CompositionModel` 与规范化排序
- [x] 单元测试覆盖 ID、Slot 矩阵、属性默认值/约束、对象标识、错误码、排序
- [x] `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 全绿
- [x] 更新 `crates/README.md`、`docs/ai/architecture-entry.md`、`working/plan.md`

## 验收

- `cargo test --workspace`：**58 passed / 0 failed**（要求 ≥ 25）；
- `cargo clippy --workspace --all-targets -- -D warnings`：通过；
- `cargo fmt --all --check`：通过；
- `composition-domain` 不依赖第三方 crate：`Cargo.lock` 只有 `composition-domain` 自身；
- `ErrorCode`：69 个变体 = 63 个验证代码 + 6 个预留代码，`from_code` 全量往返，层级/严重级别有测试；
- `ObjectId` 与 `SourceDigest` 格式符合 `docs/spec/`；
- `CompositionModel::normalize` 覆盖模块、实例、Binding、Connection、依赖、诊断与 `module_roots` 排序。

## 结果 / 遗留问题

- 结果：根 `Cargo.toml` / `Cargo.lock`；`crates/composition-domain` 的 11 个模块（ids、schema、
  value、contract、module、composition、diagnostic、revision、model、error、error_code）；
  58 项单元测试。
- 关键决策：
  - 单个 `composition-domain` crate 起步，其余 crate 在 006 / 007 / M2+ 按依赖方向加入；
  - 只用标准库，不引入 serde：`ConfigValue` 自建，序列化留到 008 的协议层；
  - 契约集合用 `BTreeMap` 保证唯一与稳定顺序；`accepts` / `enum_values` 在 normalize 时排序；
  - `DomainError` 携带 `ErrorCode`，避免领域错误与诊断代码各写一套；
  - 预留的 M2/M3 代码也进入枚举，避免后续改名。
- 遗留：
  - 还没有 TOML 读取、Source Map 与 `source_digest` 计算（006）；
  - `Project` / `Composition` 的跨对象校验（root、引用、结构）留给 007；本卡只做契约自洽；
  - `SchemaUnsupportedFeature` 尚未被使用，保留给后续显式不支持项。
- 下一步：006（TOML 加载与 Source Map），进入前创建任务卡与 `composition-toml` crate。
