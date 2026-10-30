---
name: Bug report / 缺陷反馈
about: Report a reproducible defect in Device Viewer / 反馈可复现的缺陷
title: "[Bug] "
labels: bug
assignees: ""
---

> **法律声明（必读）**
>
> 1. Device Viewer 的完整著作权归原作者独有。本项目源代码可见，但**不属于 OSI 认可的开源软件**，不适用 MIT、Apache-2.0、GPL、BSD、MPL 等任何通用开源许可证。
> 2. 个人、非营利组织可免费本地使用、学习研究；不得以“研究”“教学”“公益”“开源”“社区”等名义为盈利组织规避本协议。
> 3. 所有盈利组织、商业团队、企业、工作室、付费开发者，未经作者事先书面授权，**禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖**。商业授权邮箱：**yanruoxinsu2@outlook.com**。
> 4. 任何个人/组织修改源代码，必须在本仓库提交 **Modify Source Issue**，写明修改位置、修改理由与变更内容；未按此报备视为侵权。
> 5. 严禁任何主体修改代码后二次公开、二次发布、二次开源、二次引流售卖；GitHub Fork 仅是平台功能，**不构成授权**，禁止 Fork 后二次发布、分发或商用；禁止借 Fork、镜像、补丁、差异文件、代码片段规避上述限制。
> 6. 查实商业侵权、未授权商用、二次发布或私自魔改分发后，作者将依据《中华人民共和国著作权法》等启动完整侵权追责流程，包含民事起诉、侵权索赔、下架全部侵权项目、公开追责。
> 7. 协议最终解释权归原作者所有；管辖法律为中华人民共和国大陆地区法律。

## 提交前检查

- [ ] 我已搜索现有 Issue，确认没有重复问题。
- [ ] 我已使用最新 Release 或最新 `main` 构建复现。
- [ ] 我已删除日志、截图、快照中的序列号、主机名、MAC 地址、IP 地址、用户名、激活信息等敏感内容。
- [ ] 我理解本 Issue 是公开的，不会粘贴任何密钥、证书、令牌或私人数据。
- [ ] 我已阅读 `LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md` 与 `DISCLAIMER.md`。

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

## 法律与隐私确认

- [ ] 我确认本 Issue 不包含任何商业授权申请；商业使用必须单独发送邮件至 **yanruoxinsu2@outlook.com** 取得书面授权。
- [ ] 我确认未获得作者书面许可前，不会将本项目或其修改版本用于商业用途、二次发布、二次开源、二次闭源、二次打包或二次分发。
- [ ] 我确认 GitHub Fork 不构成授权，不会将 Fork 作为独立项目二次发布、分发或商用。
- [ ] 我理解提交 Bug 报告不代表作者放弃任何权利，也不构成对衍生作品的认可。
