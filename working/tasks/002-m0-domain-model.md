# 002 — M0 领域模型与术语冻结

- 状态：done
- 目标：冻结 v0.1 的领域对象、关系语义与不变量，产出 `docs/spec/domain-model-v1.md` 与
  `docs/glossary.md`，让两个实现者仅凭规范就能对同一个项目模型做出一致判断。
- 授权：Modify（仅 `docs/`）
- 变更声明：无（仅文档；范围示例见 `changes/example.yaml`）

> 授权审计（2026-11-27）：审计建议 **Rearchitect**（领域模型/对象关系冻结）；原始授权字段保持不变。见 [审计记录](../../docs/reviews/2026-11-27-authorization-audit-record.md)。

## 背景

设计基线 §3 列出了 9 类对象，但只给了定义与职责，没有给出最小字段、ID 规则和关系不变量。
003 要冻结 TOML 字段，必须先有一份不依赖 TOML 语法的领域模型，否则 Schema 会把偶然的
措辞固化成格式。设计基线 §15 中“模块版本策略 / 模块来源 / Slot 基数 / Dependency”等决策
也依赖这里的语义。

本卡只冻结**语义**，不决定 TOML 字段名、默认值和错误代码（那是 003 的范围）。

## 清单

- [x] 写术语表 `docs/glossary.md`：中文 / 英文 / 定义 / 常见误用
- [x] 为 Project、Module Definition、Instance、Binding、Dependency、Port、Connection、Diagnostic、Revision 给出最小字段与不变量
- [x] 画对象关系图（Mermaid）：哪些引用哪些；哪些是同层关系、哪些是包含
- [x] 写 Binding / Dependency / Connection 三者对照表，每类至少 2 个正例、1 个反例
- [x] 冻结 composite / primitive 的判定规则；明确 Adapter 在 v0.1 只是显式模块角色
- [x] 冻结 ID 与版本规则：全局唯一性范围、稳定 ID、模块 `version` 与 `schema_version` 的区别
- [x] 冻结 Diagnostic 的字段、severity 与“稳定代码”原则
- [x] 冻结 Revision 语义：内容摘要 + 提交序列，不依赖文件修改时间
- [x] 明确 v0.1 unsupported 清单（ADR-0002）
- [x] 与设计基线 §3 / §15 的差异写成 ADR，或在文档中逐条注明

## 验收

- 术语表中每个词只有一种定义，且有反例；已交付 5 组术语。
- 每个对象的不变量都能被 003 映射成具体校验：48 条编号不变量均标注校验层与首个错误代码。
- 三个关系的对照表能判定歧义案例（“A 依赖 B 但 B 不是 A 的子节点”等），含 3 个补充反例。
- 未引入设计基线之外的领域对象；差异（模块文件 `schema_version`、依赖 provider 映射、
  依赖环、单 root 字段、`object` 属性推迟、Connection 范围、Revision 摘要、`.designer` 隔离）
  已写入 domain §9，并由 ADR-0001 / ADR-0002 记录。
- 文档中的语义能支撑设计基线 §15 的前 7 项决策，具体冻结在 003 完成。

## 结果 / 遗留问题

- 结果：`docs/spec/domain-model-v1.md`、`docs/glossary.md`、`docs/adr/0001-domain-and-schema-freeze.md`、
  `docs/adr/0002-v01-unsupported.md`；9 类对象、48 条不变量、11 种 `object_id` 形式。
- 关键决策：单父 Binding 树；依赖 provider 可在实例级显式映射；依赖环允许；
  Revision 用 `seq` + 文件字节摘要；`.designer/**` 不进入 Core。
- 遗留：术语的英文命名仍可在 M1 实现时微调，但中文定义与不变量不变。
- 下一步：003（TOML 与契约 Schema 冻结）。
