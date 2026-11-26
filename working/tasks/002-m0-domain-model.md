# 002 — M0 领域模型与术语冻结

- 状态：todo
- 目标：冻结 v0.1 的领域对象、关系语义与不变量，产出 `docs/spec/domain-model-v1.md` 与
  `docs/glossary.md`，让两个实现者仅凭规范就能对同一个项目模型做出一致判断。
- 授权：Modify（仅 `docs/`）
- 变更声明：无（仅文档；范围示例见 `changes/example.yaml`）

## 背景

设计基线 §3 列出了 9 类对象，但只给了定义与职责，没有给出最小字段、ID 规则和关系不变量。
003 要冻结 TOML 字段，必须先有一份不依赖 TOML 语法的领域模型，否则 Schema 会把偶然的
措辞固化成格式。设计基线 §15 中“模块版本策略 / 模块来源 / Slot 基数 / Dependency”等决策
也依赖这里的语义。

本卡只冻结**语义**，不决定 TOML 字段名、默认值和错误代码（那是 003 的范围）。

## 清单

- [ ] 写术语表 `docs/glossary.md`：中文 / 英文 / 定义 / 常见误用
- [ ] 为 Project、Module Definition、Instance、Binding、Dependency、Port、Connection、Diagnostic、Revision 给出最小字段与不变量
- [ ] 画对象关系图（Mermaid）：哪些引用哪些；哪些是同层关系、哪些是包含
- [ ] 写 Binding / Dependency / Connection 三者对照表，每类至少 2 个正例、1 个反例
- [ ] 冻结 composite / primitive 的判定规则；明确 Adapter 在 v0.1 只是显式模块角色
- [ ] 冻结 ID 与版本规则：全局唯一性范围、稳定 ID、模块 `version` 与 `schema_version` 的区别
- [ ] 冻结 Diagnostic 的字段、severity 与“稳定代码”原则
- [ ] 冻结 Revision 语义：内容摘要 + 提交序列，不依赖文件修改时间
- [ ] 明确 v0.1 unsupported 清单（写进文档，不用隐式行为代替）
- [ ] 与设计基线 §3 / §15 的差异写成 ADR，或在文档中逐条注明

## 验收

- 术语表中每个词只有一种定义，且有反例；
- 每个对象的不变量都能被 003 映射成具体校验（可定位到对象与字段）；
- 三个关系的对照表能判定歧义案例（“A 依赖 B 但 B 不是 A 的子节点”等）；
- 未引入设计基线之外的领域对象；若必须引入，已按 Rearchitect 报告阻塞；
- 文档中的语义能支撑设计基线 §15 的前 7 项决策（版本、来源、拆分、Slot、Port、Dependency、默认值），具体冻结在 003 完成。

## 结果 / 遗留问题

- <完成后回写：结果、未覆盖项、下一步建议>
