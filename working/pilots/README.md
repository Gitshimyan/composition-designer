# Pilots

真实项目试点与 Schema 复核材料。试点只读外部项目，不复制其源码；这里是 Composition Designer
可验证的映射声明。

| 试点 | 来源 | 状态 | 用途 |
|---|---|---|---|
| [Glide Task + Terminal](./glide-task-terminal/README.md) | `E:\Work.Projects\Glide` | 通过（021） | 7 模块 / 7 实例 / 6 Binding / 6 Connection / 3 Dependency |

验证：

```powershell
target\debug\composition-cli.exe validate working\pilots\glide-task-terminal --json
target\debug\composition-cli.exe resolve working\pilots\glide-task-terminal
```
