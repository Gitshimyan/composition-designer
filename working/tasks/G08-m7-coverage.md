# G08 — M7 覆盖审计与 UW 回收

- 状态：done
- coverage_status：complete_with_deferred_items
- open_items：UW-031
- 目标：在 M7 Source 完成后回查 UW 登记：关闭 UW-011，登记模块文件 Source 编辑为 UW-031。
- authorization：
  - required：modify
  - approval：
    - modify：default
- 来源：M7 029–031 完成；用户要求继续
- 变更声明：`changes/G08-m7-coverage.yaml`

## 范围

1. 关闭 UW-011（Composition Source 编辑/冲突）；
2. 新增 UW-031（模块文件 Source 编辑，deferred）；
3. 更新登记册计数、plan 日志、M7 验收记录的未实现边界；
4. 不改代码/Schema/命令。

## 验收

- UW-011 有实现证据与关闭日期；
- UW-031 有状态、影响、授权与后续处理；
- 登记册与 plan 统计一致（31/11/5/3/12）；
- 无代码变更。

## 结果 / 遗留问题

- 结果：
  - UW-011 implemented（029–031）；
  - UW-031 deferred（模块文件编辑，Rearchitect + Extend）；
  - 登记册 31 项：11 关闭、5 待决策、3 accepted、12 延期；
  - M7 验收记录补充模块文件边界。
- 下一步：S1 Schema v2.0（032–035），需逐卡 Rearchitect 授权。