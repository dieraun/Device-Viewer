---
name: Bug report / 缺陷反馈
about: Report a reproducible defect in Device Viewer / 反馈可复现的缺陷
title: "[Bug] "
labels: bug
assignees: ""
---

## 提交前检查

- [ ] 我已搜索现有 Issue，确认没有重复问题。
- [ ] 我已使用最新 Release 或最新 `main` 构建复现。
- [ ] 我已删除日志、截图、快照中的序列号、主机名、MAC 地址、IP 地址、用户名、激活信息等敏感内容。
- [ ] 我理解本 Issue 是公开的，不会粘贴任何密钥、证书、令牌或私人数据。

## 环境信息

| 项目 | 内容 |
| --- | --- |
| Device Viewer 版本 | 例如 `0.1.0`，或 commit SHA |
| 操作系统与版本 | 例如 Windows 11 23H2 / Ubuntu 24.04 / macOS 14.5 |
| CPU 架构 | x86_64 / aarch64 |
| 安装方式 | Release 压缩包 / 源码构建 / 其他 |
| 是否便携模式 | 是（`--portable`）/ 否 |
| 是否管理员/root | 是 / 否 |
| 桌面环境 | Windows 桌面 / X11 / Wayland / 无图形环境 |
| GPU 与驱动版本 | 例如 NVIDIA 550.54 / Mesa 24.0 |
| 构建 feature | 默认 / `vulkan,opencl,sensors` / 其他 |

## 问题描述

<!-- 用一两句话说明发生了什么。 -->

## 复现步骤

1.
2.
3.

**复现频率：** 必现 / 偶现（约 __ 次中出现 __ 次）

## 预期行为

<!-- 你期望发生什么？ -->

## 实际行为

<!-- 实际发生了什么？请附上完整错误文本，不要只写“打不开”。 -->

## 日志与截图

<!--
日志默认位置：
- Windows: %APPDATA%\DeviceViewer\device-viewer.log
- macOS:   ~/Library/Application Support/DeviceViewer/device-viewer.log
- Linux:   $XDG_CONFIG_HOME/device-viewer/device-viewer.log 或 ~/.config/device-viewer/device-viewer.log
- 便携模式：可执行文件旁的 device-viewer.log

粘贴前请脱敏。不要上传完整 .dvsnap，除非已删除设备标识。
-->

```text
在此粘贴脱敏后的日志
```

## CLI 复现（如适用）

```text
DeviceViewer --cli --only <module> --out <file> --quiet
```

请同时提供 `--version` 输出与退出码。

## 影响范围

- [ ] 启动失败 / 崩溃
- [ ] 单个硬件面板无法显示
- [ ] 数据明显错误
- [ ] 导出或快照异常
- [ ] 性能或卡顿
- [ ] 界面 / 国际化 / 主题
- [ ] 其他

**严重程度：** 阻塞 / 严重 / 一般 / 轻微

## 附加信息

<!-- 最近的变更、特殊硬件、虚拟机、远程桌面、杀毒软件、代理等。 -->

> 提交前请确认已脱敏，并遵守 [行为准则](CONTRIBUTING.md)。
