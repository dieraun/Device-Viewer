# Device Viewer 1.0.0

首个正式版本 · First stable release

---

## 中文

Device Viewer 1.0.0 是项目的首个正式版本。它是一款使用 Rust 与 eframe/egui 构建的跨平台硬件信息查看工具，面向需要快速确认本机硬件、驱动接口、传感器状态和图形能力的开发者与高级用户。

### 本版亮点

- **原生跨平台**：Windows x86_64、macOS x86_64 / Apple Silicon、Linux x86_64 使用同一套 Rust 代码构建。
- **GUI 与 CLI 双模式**：桌面界面适合浏览和对比；命令行支持 `--out`、`--only`、`--quiet`、`--snapshot`，便于脚本化和归档。
- **采集与界面解耦**：`device_query` 只负责采集，UI 只读取快照结构，不直接触碰硬件。
- **异步并行采集**：各硬件模块并行执行，每个模块拥有独立超时与取消令牌；单个模块失败不会中断整个采集流程。
- **完整硬件视图**：CPU、内存、PCIe、Vulkan、OpenGL、OpenCL、NPU、USB、存储、主板/BIOS、显示器、DirectX、Metal、OpenVX、系统与网络、音频、传感器、内存带宽。
- **GPU 实时曲线**：Vulkan 面板展示显存占用率与 GPU 利用率的历史曲线，并列出显存堆与扩展。
- **存储健康**：读取 NVMe / SATA SMART 健康、温度、通电时间、写入量、坏块、介质错误等信息；缺少 `smartctl` 时使用 sysinfo 作为降级数据源。
- **传感器面板**：温度、风扇转速、核心电压集中展示；非特权运行时会明确提示权限限制。
- **快照对比**：保存 `.dvsnap`，比较新增、移除和变化的字段；支持标签、拖放导入和双击打开。
- **多格式导出**：TXT、Markdown、JSON，支持预览、复制 Markdown 表格和保存到文件。
- **Vulkan 开发面板**：生成 C 与 Rust/ash 初始化代码片段，并解释常用扩展用途。
- **中英文界面**：中文与英文可即时切换，自动安装可用 CJK 字体。
- **明暗主题与便携模式**：便携模式将配置、日志与快照保存在可执行文件旁边。
- **失败可见**：模块错误分为“警告”和“错误”，权限不足被视为非阻塞警告，不会伪装成成功。

### 架构

```text
device_query (async collectors, cancellation, timeout, serde models)
        │
        ▼
HardwareSnapshot ──── state (config, snapshots, portable paths, compare)
        │
        ▼
ui / export / cli (read-only views, no hardware queries)
```

这种分层带来的直接好处是：采集可以单独测试；界面切换页面不会阻塞硬件；导出与 CLI 复用同一份快照；模块故障被隔离在 `failures` 列表中。

### 下载（Windows）

| 平台 | 架构 | 文件 |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.0.0.zip` |

本版本仅提供 Windows x86_64 预编译包；macOS 与 Linux 可从同一份源码构建，本 Release 不附带这两个平台的预编译二进制。

压缩包包含 `DeviceViewer.exe`、`README.md`、`LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`、平台说明与发行说明，并附带 SHA-256 校验文件。

### 运行说明

- 需要 64 位 Windows 10/11 或 Windows Server 2016 及以上版本。
- 需要 Microsoft Visual C++ 2015–2022 运行库（x64）。
- 读取 SMART、部分传感器或某些设备权限受限的信息时，建议以管理员身份运行；权限不足会显示为非阻塞警告。
- OpenVX 为可选编译特性，1.0.0 的官方二进制未启用；面板会明确显示“未启用”，而不是伪造设备信息。

### 升级说明

这是首个稳定版本，配置结构使用 `schema_version` 管理。旧版本配置与快照会尽可能兼容读取；未知字段会被忽略，未知错误类型会降级为 `unknown`，不会导致解析崩溃。

### 已知限制

- 硬件、固件、驱动与操作系统版本差异会影响可用字段；缺失数据会显示为不可用，而不是猜测值。
- 虚拟机、远程桌面和部分服务器平台可能不暴露温度、风扇或 GPU 遥测。
- GPU 利用率与显存遥测依赖驱动和平台接口，数值可能与其他工具存在采样差异。
- 内存带宽为只读基准测试，不代表网络带宽或磁盘吞吐。

### 许可与商业使用

本项目为**源码可见的专有软件**，不是 OSI 开源项目。个人与非营利组织可免费本地使用、学习研究。任何公司、商业团队、工作室或付费开发者，未经作者书面授权，不得使用、复制、修改、衍生、二次开源、二次闭源、打包、分发、发布、售卖、集成进商业项目、SaaS、付费工具或课程。商业授权请联系：**yanruoxinsu2@outlook.com**。

修改源代码必须在项目仓库提交 Modify Source Issue 报备；未报备视为侵权。GitHub Fork 仅是平台功能，不构成授权，禁止 Fork 后二次发布、分发或商用。

---

## English

Device Viewer 1.0.0 is the first stable release of a cross-platform hardware information viewer built with Rust and eframe/egui. It is aimed at developers and power users who need a fast, readable view of local hardware, driver interfaces, sensor state, and graphics capabilities.

### Highlights

- **Native cross-platform builds**: one Rust codebase for Windows x86_64, macOS x86_64 / Apple Silicon, and Linux x86_64.
- **GUI and CLI**: a desktop UI for browsing and comparison; a CLI with `--out`, `--only`, `--quiet`, and `--snapshot` for scripting and archiving.
- **Decoupled collection and presentation**: `device_query` collects data; the UI only reads snapshot structures and never queries hardware directly.
- **Async parallel collection**: every module runs concurrently with its own timeout and cancellation token. One failing module cannot abort the whole pass.
- **Broad hardware coverage**: CPU, memory, PCIe, Vulkan, OpenGL, OpenCL, NPU, USB, storage, motherboard/BIOS, displays, DirectX, Metal, OpenVX, system and network, audio, sensors, and memory bandwidth.
- **Live GPU curves**: the Vulkan panel plots VRAM usage and GPU utilization over time, with memory heaps and extensions.
- **Storage health**: NVMe/SATA SMART health, temperature, power-on hours, total written bytes, bad blocks, and media errors, with a sysinfo fallback when `smartctl` is absent.
- **Sensor panel**: temperatures, fan speeds, and core voltages in one place, with explicit permission warnings when running unprivileged.
- **Snapshot comparison**: save `.dvsnap` files, diff added, removed, and changed fields, attach tags, drag-and-drop imports, and double-click to compare.
- **Multi-format export**: TXT, Markdown, and JSON with preview, Markdown table copy, and save-to-file.
- **Vulkan developer panel**: generated C and Rust/ash initialization snippets plus explanations of common extensions.
- **Chinese and English UI**: switch languages at runtime, with automatic CJK font installation when available.
- **Light/dark themes and portable mode**: portable mode keeps configuration, logs, and snapshots beside the executable.
- **Visible failure handling**: module errors are classified as warnings or errors. Permission problems are non-blocking warnings and are never presented as success.

### Architecture

```text
device_query (async collectors, cancellation, timeout, serde models)
        │
        ▼
HardwareSnapshot ──── state (config, snapshots, portable paths, compare)
        │
        ▼
ui / export / cli (read-only views, no hardware queries)
```

The practical benefits: collectors are testable in isolation; page switches never block on hardware; export and CLI reuse the same snapshot; module failures stay contained in the `failures` list.

### Downloads (Windows)

| Platform | Architecture | File |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.0.0.zip` |

This release provides a Windows x86_64 prebuilt package only. macOS and Linux can be built from the same source; no prebuilt binaries for those platforms are attached to this release.

The archive contains `DeviceViewer.exe`, `README.md`, `LICENSE`, `LEGAL_STATEMENT.md`, `PRIVACY.md`, `DISCLAIMER.md`, platform notes, and these release notes, plus a SHA-256 checksum file.

### Running

- 64-bit Windows 10/11 or Windows Server 2016 and newer.
- Microsoft Visual C++ 2015–2022 runtime (x64) is required.
- Administrator rights are recommended for SMART, some sensors, and devices with restricted permissions. Permission problems appear as non-blocking warnings.
- **OpenVX** is an optional compile-time feature and is not enabled in the official 1.0.0 binary. The panel reports it as unavailable instead of inventing device data.

### Upgrade notes

This is the first stable release. Configuration uses `schema_version`. Older configuration and snapshot files are read on a best-effort basis; unknown fields are ignored and unknown error kinds degrade to `unknown` rather than failing deserialization.

### Known limitations

- Hardware, firmware, driver, and OS differences affect which fields are available. Missing data is shown as unavailable rather than guessed.
- Virtual machines, remote desktop sessions, and some server platforms may not expose temperature, fan, or GPU telemetry.
- GPU utilization and VRAM telemetry depend on driver and platform interfaces and may differ from other tools because of sampling.
- The memory bandwidth figure is a read-only benchmark, not network throughput or disk I/O.

### License and commercial use

This is **source-visible proprietary software**, not an OSI open-source project. Individuals and non-profit organizations may use it locally for free and study the source. Companies, commercial teams, studios, and paid developers may not use, copy, modify, create derivatives from, re-open-source, re-close, package, distribute, publish, sell, integrate into commercial projects, offer as SaaS, or include in paid tools or courses without prior written permission from the author. Commercial licensing: **yanruoxinsu2@outlook.com**.

Any modification must be reported in a Modify Source Issue in the project repository; failure to report is infringement. A GitHub Fork is a platform feature, not an authorization; do not publish, distribute, or commercially use a fork.
