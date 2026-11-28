# Glide Task + Terminal Pilot (Schema v2)

- 来源：`working/pilots/glide-task-terminal/`（v1 试点）
- 性质：同一 Glide 切片的 v2.0 声明；不复制 Glide 源码
- 目的：验收接口身份（G1）与 service scope（G2）

## 1. v2 映射变化

- `task-runner`、`terminal-web`、`conpty-session` 不再通过 root 的人工 `runtime` Slot 挂在结构树上，
  而是 `scope = "service"` 的实例，可以从 root 不可达；
- 依赖改用接口身份：
  - task-panel → `glide.task.runner`
  - task-runner → `glide.terminal.session`
  - terminal-panel → `glide.terminal.web`
- 结构树只保留真实 UI 层级：app → workbench → task-panel / terminal-panel；
- Port / Connection 与 v1 试点保持一致。

## 2. 结果

```text
instances    7 (4 structure + 3 service)
bindings     3
connections  6
dependencies 3
modules      7
diagnostics  0
```

## 3. 验证

```powershell
target\debug\composition-cli.exe validate working\pilots\glide-task-terminal-v2 --json
target\debug\composition-cli.exe resolve working\pilots\glide-task-terminal-v2
```

## 4. 边界

- 项目级 Profile（G3）仍未表达，属 v2.1 / S2；
- 接口只有身份，没有方法/Port 形状（UW-032）；
- v1 试点保留，用于 v1 回归；本目录只用于 v2 验收。
