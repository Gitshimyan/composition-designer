# 编辑错误场景（M2）

这些场景在 M2 的编辑命令 Schema 冻结后补具体请求载荷；当前先固化前置条件与预期结果。
基座项目统一使用 [`../minimal-project`](../minimal-project)。

| 场景 | 前置 | 操作 | 预期 |
|---|---|---|---|
| 事务失败 | 读取 `minimal-project` 当前 Revision | 把 `hello` 的模块改为不存在的 `ghost.widget` | 候选模型验证失败；文件与 Revision 不变；返回 `edit-transaction-failed` |
| Revision 过期 | 会话 A 读取 Revision | 会话 B 提交一次修改；A 用旧 `base_revision` 提交 | 返回 `edit-revision-conflict`；不覆盖 B 的修改 |
| 重复 request ID | 已提交一次请求 `request_id = "r-1"` | 用同一 `request_id` 再提交 | 返回 `edit-duplicate-request`；不重复产生副作用 |

约束：

- 失败事务不得留下部分写入；
- 冲突时必须返回最新 Revision；
- 重试必须幂等。
