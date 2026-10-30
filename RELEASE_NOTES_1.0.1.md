# Device Viewer 1.0.1

OpenGL 修复版本 · OpenGL hotfix release

2026-10-30

---

## 中文

Device Viewer 1.0.1 是 1.0.0 的维护版本，只修复 Windows GUI 的 OpenGL 信息采集问题，不改变采集架构、配置格式或快照格式。

### 修复内容

- **OpenGL 面板恢复可用**：1.0.0 的后台采集线程没有 OpenGL 渲染上下文，OpenGL 区域始终返回“不可用”。1.0.1 在 GUI 渲染线程通过 eframe 的实时 glow 上下文读取 `GL_VENDOR`、`GL_RENDERER`、`GL_VERSION`、GLSL 版本、纹理限制与扩展列表。
- **修复 Core Profile 崩溃**：OpenGL 3.0 及以上的 Core Profile 已移除单一的 `GL_EXTENSIONS` 字符串。1.0.1 改用 `glGetStringi` 逐项读取扩展，避免读取空指针导致程序退出；OpenGL 2.x 仍保留旧接口。
- **管理员权限说明**：OpenGL 信息不需要管理员权限。SMART、部分传感器和受限设备访问才可能需要提权；权限不足仍以非阻塞警告显示。

### 下载（Windows）

| 平台 | 架构 | 文件 |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.0.1.zip` |

本版本仅提供 Windows x86_64 预编译包；macOS 与 Linux 可从同一份源码构建，本 Release 不附带这两个平台的预编译二进制。

压缩包包含 `DeviceViewer.exe`、`README.md`、`LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`、平台说明与发行说明，并附带 SHA-256 校验文件。

### 升级说明

- 直接替换旧版 `DeviceViewer.exe`，或使用 1.0.1 压缩包完整覆盖。
- 配置、日志与 `.dvsnap` 快照格式未变化，1.0.0 的配置与快照可继续读取。
- 便携模式仍将配置、日志与快照保存在可执行文件旁边。

### 运行要求

- 64 位 Windows 10/11 或 Windows Server 2016 及以上版本。
- Microsoft Visual C++ 2015–2022 运行库（x64）。
- 读取 SMART、部分传感器或受限设备信息时建议以管理员身份运行；权限不足会显示为非阻塞警告。
- OpenVX 为可选编译特性，官方 Windows 二进制未启用；面板会明确显示“未启用”。

### 已知限制

- 硬件、固件、驱动与操作系统版本差异会影响可用字段；缺失数据会显示为不可用，而不是猜测值。
- 虚拟机、远程桌面和部分服务器平台可能不暴露温度、风扇或 GPU 遥测。
- 不同 OpenGL 驱动可能报告不同的扩展集合；这是驱动能力差异，不是程序错误。
- 内存带宽为只读基准测试，不代表网络带宽或磁盘吞吐。

### 许可与商业使用

本项目为**源码可见的专有软件**，不是 OSI 开源项目。个人与非营利组织可免费本地使用、学习研究。任何公司、商业团队、工作室或付费开发者，未经作者书面授权，不得使用、复制、修改、衍生、二次开源、二次闭源、打包、分发、发布、售卖、集成进商业项目、SaaS、付费工具或课程。商业授权请联系：**yanruoxinsu2@outlook.com**。

修改源代码必须在项目仓库提交 Modify Source Issue 报备；未报备视为侵权。GitHub Fork 仅是平台功能，不构成授权，禁止 Fork 后二次发布、分发或商用。

---

## English

Device Viewer 1.0.1 is a maintenance release for 1.0.0. It fixes OpenGL information collection in the Windows GUI only; the collection architecture, configuration format, and snapshot format are unchanged.

### Fixes

- **OpenGL panel restored**: the 1.0.0 background collector had no OpenGL rendering context, so the OpenGL section always reported unavailable. 1.0.1 reads `GL_VENDOR`, `GL_RENDERER`, `GL_VERSION`, GLSL version, texture limits, and extensions from eframe's live glow context on the GUI render thread.
- **Core Profile crash fixed**: OpenGL 3.0+ Core Profiles removed the monolithic `GL_EXTENSIONS` string. 1.0.1 enumerates extensions with `glGetStringi`, avoiding a null-pointer read that could terminate the application. OpenGL 2.x keeps the legacy path.
- **Administrator note**: OpenGL information does not require administrator rights. SMART, some sensors, and restricted devices may still need elevation; permission problems remain non-blocking warnings.

### Downloads (Windows)

| Platform | Architecture | File |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.0.1.zip` |

This release provides a Windows x86_64 prebuilt package only. macOS and Linux can be built from the same source; no prebuilt binaries for those platforms are attached to this release.

The archive contains `DeviceViewer.exe`, `README.md`, `LICENSE`, `LEGAL_STATEMENT.md`, `PRIVACY.md`, `DISCLAIMER.md`, platform notes, and these release notes, plus a SHA-256 checksum file.

### Upgrade notes

- Replace the old `DeviceViewer.exe`, or overwrite the installation with the 1.0.1 archive.
- Configuration, logs, and `.dvsnap` snapshot formats are unchanged. 1.0.0 configuration and snapshots remain readable.
- Portable mode still keeps configuration, logs, and snapshots beside the executable.

### Requirements

- 64-bit Windows 10/11 or Windows Server 2016 and newer.
- Microsoft Visual C++ 2015–2022 runtime (x64).
- Administrator rights are recommended for SMART, some sensors, and restricted devices. Permission problems appear as non-blocking warnings.
- **OpenVX** is an optional compile-time feature and is not enabled in the official Windows binary. The panel reports it as unavailable.

### Known limitations

- Hardware, firmware, driver, and OS differences affect which fields are available. Missing data is shown as unavailable rather than guessed.
- Virtual machines, remote desktop sessions, and some server platforms may not expose temperature, fan, or GPU telemetry.
- Different OpenGL drivers report different extension sets; this reflects driver capability, not an application error.
- The memory bandwidth figure is a read-only benchmark, not network throughput or disk I/O.

### License and commercial use

This is **source-visible proprietary software**, not an OSI open-source project. Individuals and non-profit organizations may use it locally for free and study the source. Companies, commercial teams, studios, and paid developers may not use, copy, modify, create derivatives from, re-open-source, re-close, package, distribute, publish, sell, integrate into commercial projects, offer as SaaS, or include in paid tools or courses without prior written permission from the author. Commercial licensing: **yanruoxinsu2@outlook.com**.

Any modification must be reported in a Modify Source Issue in the project repository; failure to report is infringement. A GitHub Fork is a platform feature, not an authorization; do not publish, distribute, or commercially use a fork.
