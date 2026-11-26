# 外部修改场景（M3）

基座项目统一使用 [`../minimal-project`](../minimal-project)。M3 实现文件监听后补操作序列。

| 场景 | 外部动作 | 预期 |
|---|---|---|
| 合法外部修改 | 把 `composition/main.toml` 中 `hello` 的 `label` 改为 `"Hi"` | 重新解析成功；模型与 Revision 更新；诊断清空 |
| 非法外部修改 | 删除 `app-content` Binding | 保留最后一个有效模型；显示 `structure-slot-cardinality`；不得用旧模型覆盖用户文件 |
| 未保存冲突 | 编辑器有未提交修改时外部改同一文件 | 明确提示冲突；不得静默覆盖任一侧 |

约束：外部修改不得绕过解析与验证；无效文件不会触发自动回滚写入。
