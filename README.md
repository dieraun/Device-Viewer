# Device Viewer

**跨平台硬件信息查看工具 · Cross-platform hardware inspector**

Rust · eframe/egui · 异步并行采集 · GUI + CLI · Windows / Linux / macOS

**当前版本 Current release: 1.0.1** · [发行说明 Release notes](RELEASE_NOTES_1.0.1.md)

> 本项目采用**定制化专有许可协议**，不是开源软件。个人与非营利组织可免费本地使用；任何商业用途必须事先获得作者书面授权。详见 [LICENSE](LICENSE) 与 [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md)。

---

## 中文

### 1. 项目简介

Device Viewer 是一款使用 Rust 编写的跨平台硬件信息查看工具。它通过统一的数据采集层读取 CPU、内存、主板、PCIe、USB、存储、显示器、传感器、网络、音频以及图形与计算运行时信息，并以原生桌面界面或命令行报告的形式呈现。

项目将**硬件采集**与**界面展示**彻底解耦：采集层只负责生成结构化、可序列化的数据；界面层只读取这些数据负责呈现。任一采集模块出现权限不足、驱动缺失、设备不支持或超时，都不会中断其他模块，也不会导致整个应用崩溃。

### 2. 核心优势

- **原生性能**：Rust 编译为本地机器码，启动快、内存占用可控，无浏览器内核与运行时解释开销。
- **异步并行采集**：各硬件模块通过 Tokio 并行执行，每项任务具有独立超时与取消信号；切换页面时旧任务可被取消，结果不会互相覆盖。
- **故障隔离**：单个模块失败仅生成一条警告或错误记录，其余模块继续完成；报告始终可用。
- **采集与 UI 解耦**：`device_query` 不依赖 `eframe/egui`，可独立测试、复用或用于批处理。
- **快照对比**：将某次硬件状态保存为 `.dvsnap`，后续可与当前状态或另一份快照逐字段比较，突出新增、移除与变化项。
- **多格式导出**：TXT、Markdown、JSON；支持 Markdown 表格复制与导出预览。
- **开发者工具**：内置 Vulkan 设备信息、扩展说明、C/Rust 代码片段与内部日志查看器。
- **中英双语与明暗主题**：界面文本、导出内容与设置项均支持中文/英文；主题可持久化。
- **便携模式**：配置、快照与日志可全部保存在可执行文件旁，适合 U 盘、离线环境与受控工作站。
- **纯本地运行**：默认不联网，不上传硬件信息、用户信息或使用数据。

### 3. 功能一览

| 模块 | 主要内容 |
| --- | --- |
| CPU | 厂商、型号、架构、物理/逻辑核心、逐核心占用与频率、全局负载 |
| 内存 | 物理内存、可用内存、交换分区、模块与容量信息 |
| 主板 / BIOS | 厂商、型号、版本、序列号、资产标签、BIOS 版本与日期 |
| PCIe | 设备地址、厂商/设备 ID、类别、子系统、驱动、链路速率与宽度 |
| Vulkan | 加载器与 API 版本、物理设备、显存堆、扩展、显存预算、实时利用率/显存曲线 |
| OpenGL | 渲染器、厂商、版本、着色语言版本、关键限制与扩展 |
| OpenCL | 平台、设备、计算单元、频率、全局/局部内存、编译器与扩展 |
| NPU / 加速器 | PCI 类别识别、运行时库与设备节点探测 |
| USB | 总线/地址、VID/PID、厂商、产品、序列号、类别、速率 |
| 存储 | 型号、固件、容量、SMART 健康、温度、通电时间、写入量、坏块、NVMe 寿命 |
| 显示器 | 分辨率、刷新率、位置、主显示器、显示适配器与显存 |
| DirectX | DXGI 适配器、厂商/设备 ID、专用与共享显存、软件/远程适配器标记 |
| Metal | macOS Metal 设备、统一内存、推荐工作集、线程组限制 |
| OpenVX | 运行时可用性与上下文创建结果 |
| 传感器 | CPU/GPU/存储/主板温度、风扇转速、核心电压 |
| 网络 | 接口、MAC、IP 地址、累计收发字节 |
| 音频 | 音频端点与厂商信息 |
| 带宽 | 本机内存读、写、拷贝带宽采样 |
| 检查清单 | 各能力可用性、权限状态、警告与失败汇总 |

> 硬件信息的可见范围取决于操作系统、驱动、固件、设备权限及虚拟化环境。Device Viewer 会在可用范围内尽力呈现；个别设备或接口可能只显示部分字段，这属于正常差异，不代表程序故障。

### 4. 架构设计

```text
┌──────────────────────────────────────────────────────────────┐
│                        main.rs                              │
│         GUI 启动 / CLI 参数 / 补全脚本 / 退出码              │
└───────────────┬──────────────────────────────┬───────────────┘
                │                              │
        ┌───────▼────────┐            ┌────────▼────────┐
        │     app.rs     │            │    export.rs     │
        │ 生命周期与调度  │            │ TXT/MD/JSON 渲染 │
        └───────┬────────┘            └────────┬────────┘
                │                              │
   ┌────────────▼───────────┐       ┌──────────▼─────────┐
   │        ui/             │       │      state.rs      │
   │ 只读取快照，不采集硬件  │       │ 配置/快照/便携模式 │
   └────────────┬───────────┘       └──────────┬─────────┘
                │                              │
        ┌───────▼──────────────────────────────▼───────┐
        │              device_query/                   │
        │  20 个独立采集模块 + 统一错误 + 调度器        │
        │  CancellationToken / 超时 / spawn_blocking   │
        └───────────────────┬──────────────────────────┘
                            │
        ┌───────────────────▼──────────────────────────┐
        │ OS API · sysinfo · pci-info · ash · OpenCL   │
        │ DXGI · SetupAPI · GDI · Metal · smartctl     │
        └──────────────────────────────────────────────┘
```

**分层原则**

1. `device_query` 是纯数据层：不引用任何 UI 类型，不执行界面逻辑。
2. `ui` 是纯展示层：只读取 `HardwareSnapshot` 与 `AppState`，不直接调用系统 API。
3. `state` 负责持久化：主题、语言、单位、窗口位置、筛选条件、收藏扩展、快照索引与便携模式。
4. `app` 负责编排：启动加载页、并行采集、页面切换取消、事件轮询、GPU 曲线采样与退出清理。
5. `export` 负责多格式输出：与 UI 共用同一份快照结构，保证界面与报告一致。

**可靠性设计**

- 每个采集模块返回统一 `QueryResult<T>`；权限不足、工具缺失等可恢复问题记为警告，不阻塞整体流程。
- 采集任务带 `CancellationToken`；超时、切页、退出均可取消。
- 阻塞式系统调用放入 `spawn_blocking`，避免占用异步执行器。
- 快照写出前执行非有限浮点清洗，避免 `NaN`/无穷值破坏 JSON。
- 快照枚举含未知值回退，旧版本可读取未来新增字段；新增字段默认可选。
- GPU 资源、Windows 设备信息集等使用 RAII/Drop 释放；退出时等待 Tokio 运行时收敛。

### 5. 跨平台支持

| 平台 | 图形界面 | CLI | 说明 |
| --- | --- | --- | --- |
| Windows | 支持 | 支持 | DXGI、GDI、SetupAPI、sysinfo 等后端 |
| Linux | 支持 | 支持 | X11/Wayland、sysfs、hwmon、DRM 等后端 |
| macOS | 支持 | 支持 | Metal、system_profiler 等后端 |

不同平台的字段覆盖并不完全一致；例如 SMART 依赖系统是否提供 `smartctl`，NVIDIA 温度依赖 `nvidia-smi`，部分传感器需要管理员/root 权限。程序会对缺失项给出提示，而不是假定所有设备都具备同一能力。

### 6. 快速开始

**环境要求**

- Rust 1.95 或更高版本（edition 2024）。
- 常规 C/C++ 构建工具与系统图形开发库。
- 可选：`smartctl`（更完整的存储健康信息）、`nvidia-smi`（NVIDIA 温度）。
- 可选：Vulkan Loader、OpenCL ICD、OpenVX SDK，取决于启用的特性。

**运行 GUI**

```bash
cargo run --release
```

**运行 CLI 并导出 Markdown**

```bash
cargo run --release -- --cli --only cpu,memory,storage --out hardware.md
```

**导出 JSON**

```bash
cargo run --release -- --out hardware.json
```

**从已有快照重新导出**

```bash
cargo run --release -- --snapshot hardware.dvsnap --out hardware.md
```

**生成补全脚本**

```bash
cargo run --release -- --completions bash
cargo run --release -- --completions zsh
cargo run --release -- --completions powershell
```

### 7. CLI 参考

```text
Device Viewer — cross-platform hardware inspector

USAGE:
    device-viewer [OPTIONS]

OPTIONS:
    --cli                 Force the batch/CLI mode (implied by --out/--only/--snapshot).
    --out <PATH>          Write TXT, Markdown or JSON, chosen from the extension.
    --only <LIST>         Comma-separated modules: cpu,memory,pcie,vulkan,opengl,
                          opencl,npu,usb,storage,motherboard,display,directx,metal,
                          openvx,system,network,audio,sensors,bandwidth.
    --snapshot <PATH>     Export an existing .dvsnap/JSON snapshot instead of querying.
    --quiet               Suppress progress and warning chatter.
    --portable            Persist configuration and snapshots beside the executable.
    --completions <SHELL> Print completion script: bash, zsh or powershell.
    --version             Print the version.
    -h, --help            Print this help.
```

- 不提供批处理参数时启动 GUI。
- `--only` 只保留指定模块；`system` 会在选择 `network` 或 `audio` 时自动保留。
- 输出格式由扩展名决定：`.txt`、`.md`、`.json`。
- 未知模块、缺失参数或无法写入文件时返回非零退出码。

### 8. 特性开关

```toml
default = ["vulkan", "opencl", "metal", "sensors"]
vulkan  = ["dep:ash"]
opencl  = ["dep:opencl3"]
metal   = ["dep:metal"]
openvx  = ["dep:openvx"]
sensors = ["sysinfo/component"]
```

- `vulkan`：Vulkan 设备、显存堆与扩展信息。
- `opencl`：OpenCL 平台与设备信息。
- `metal`：macOS Metal 设备信息。
- `openvx`：OpenVX 上下文可用性；需要本机 OpenVX SDK 与 libclang，默认关闭。
- `sensors`：温度、风扇与电压传感器。

构建最小版本：

```bash
cargo run --release --no-default-features
```

### 9. 测试与质量

```bash
cargo fmt --all -- --check
cargo check --tests
cargo test
cargo test --no-default-features
```

当前测试覆盖：

- 格式化：二进制/十进制单位、频率、时长。
- 错误模型：权限不足为警告、超时为错误、JSON 往返。
- 采集模块：CPU、内存、PCIe、存储、主板、显示器、系统、传感器、USB 的结构化结果。
- 调度：逐模块超时、取消信号、故障隔离。
- 快照：可选字段序列化、默认值、未知字段/未知枚举前向兼容、非有限值清洗。
- 状态：快照文件名唯一性、路径穿越拦截、便携模式根目录切换、时间戳渲染。

### 10. 数据与隐私

- 默认不发起任何网络请求，不上传硬件信息、快照、日志或使用统计。
- 配置、快照、日志均保存在本机；便携模式下位于可执行文件旁。
- 快照可能包含主机名、序列号、MAC 地址、IP 地址等设备标识。分享前请自行审查。
- 详细说明见 [PRIVACY.md](PRIVACY.md)。

### 11. 许可证与商业授权

本项目采用**定制化专有许可协议**，不是 MIT、Apache-2.0、GPL、BSD 或任何 OSI 认可的开源许可证。

1. 本项目完整著作权归原作者独有。
2. 个人、非营利组织：可以免费本地使用、学习研究。
3. 所有盈利组织、商业团队、企业、工作室、付费开发者：禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖。所有商业用途必须提前发送邮件 yanruoxinsu2@outlook.com 获取作者书面授权，未经授权全部属于侵权。
4. 任何个人/组织修改源代码，必须在项目仓库提交 Issue 写明修改内容，不报备视为侵权。
5. 严禁任何主体修改代码后二次公开、二次发布、二次开源、二次引流售卖。
6. 一旦查实商业侵权、二次发布、未授权商用、私自魔改分发行为，作者将直接依据《中华人民共和国著作权法》启动完整法律追责流程，包含民事起诉、侵权索赔、下架全部侵权项目、公开追责。
7. 协议最终解释权归原作者所有；管辖法律：中华人民共和国大陆地区法律。

完整条款以 [LICENSE](LICENSE) 为准；法律边界、侵权后果与维权方式见 [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md)。

### 12. 免责声明

本软件按“现状”提供，不保证硬件信息、SMART 数据、传感器读数或兼容性完全准确。详细免责条款见 [DISCLAIMER.md](DISCLAIMER.md)。

### 13. 联系

- 商业授权：**yanruoxinsu2@outlook.com**
- 修改报备：请在项目仓库提交 Issue，写明修改目的、范围、文件与行为变化。

---

## English

### 1. Overview

Device Viewer is a cross-platform hardware inspector written in Rust. A unified collection layer reads CPU, memory, motherboard, PCIe, USB, storage, display, sensor, network, audio, graphics and compute-runtime information, then presents it through a native desktop interface or a command-line report.

The project keeps **hardware collection** and **presentation** strictly separated. Collectors produce structured, serializable data; the UI only reads that data. If one collector encounters missing permissions, a missing driver, an unsupported device or a timeout, the other collectors still finish and the application does not crash.

### 2. Highlights

- **Native performance** — compiled to machine code; fast startup, predictable memory use, no embedded browser engine.
- **Asynchronous parallel collection** — Tokio runs collectors concurrently with per-module timeouts and cancellation. Page changes cancel stale work and cannot overwrite newer results.
- **Failure isolation** — a failed module produces a warning or error entry while the rest of the report remains available.
- **Decoupled architecture** — `device_query` has no dependency on `eframe/egui` and can be tested or reused independently.
- **Snapshot comparison** — save a `.dvsnap`, then diff it against the current state or another snapshot with added/removed/changed highlighting.
- **Multi-format export** — TXT, Markdown and JSON, with preview and Markdown table copy.
- **Developer tools** — Vulkan device details, extension descriptions, C/Rust snippets and an internal log viewer.
- **Bilingual UI** — Chinese and English, including exported reports; dark and light themes.
- **Portable mode** — keep configuration, snapshots and logs beside the executable for USB drives and offline workstations.
- **Local by design** — no network access, no upload of hardware or user data, no telemetry.

### 3. Feature Matrix

| Area | What is shown |
| --- | --- |
| CPU | Vendor, model, architecture, physical/logical cores, per-core usage and frequency, global load |
| Memory | Total, used, available, swap, module and capacity information |
| Motherboard / BIOS | Vendor, model, version, serial, asset tag, BIOS version and date |
| PCIe | Address, vendor/device ID, class, subsystem, driver, link speed and width |
| Vulkan | Loader/API version, physical devices, memory heaps, extensions, memory budget, live utilization/VRAM curves |
| OpenGL | Renderer, vendor, version, shading language, key limits and extensions |
| OpenCL | Platforms, devices, compute units, clocks, global/local memory, compiler and extensions |
| NPU / accelerators | PCI class detection, runtime libraries and device nodes |
| USB | Bus/address, VID/PID, vendor, product, serial, class, speed |
| Storage | Model, firmware, capacity, SMART health, temperature, power-on time, written bytes, bad blocks, NVMe wear |
| Display | Resolution, refresh rate, position, primary flag, adapters and VRAM |
| DirectX | DXGI adapters, vendor/device IDs, dedicated/shared memory, software/remote flags |
| Metal | macOS Metal devices, unified memory, recommended working set, threadgroup limits |
| OpenVX | Runtime availability and context creation result |
| Sensors | CPU/GPU/storage/motherboard temperatures, fan RPM, core voltage |
| Network | Interfaces, MAC, IP addresses, cumulative RX/TX bytes |
| Audio | Audio endpoints and vendors |
| Bandwidth | Local memory read, write and copy samples |
| Checklist | Capability availability, permission state, warnings and failures |

> What can be observed depends on the operating system, drivers, firmware, device permissions and virtualization. Device Viewer presents as much as the environment exposes; partial fields on some devices are normal and do not indicate a program defect.

### 4. Architecture

```text
main.rs        GUI startup / CLI parsing / completions / exit codes
app.rs         lifecycle, query scheduling, event polling, shutdown
export.rs      TXT / Markdown / JSON rendering
ui/            read-only presentation; no hardware queries
state.rs       settings, snapshots, portable paths, favorites
device_query/  20 collectors, unified errors, cancellation and timeouts
```

Design rules:

1. `device_query` is a pure data layer with no UI types.
2. `ui` never calls operating-system APIs directly.
3. `state` persists theme, language, units, window rect, filters, favorite extensions, snapshots and portable mode.
4. `app` orchestrates loading, collection, cancellation, GPU sampling and cleanup.
5. `export` renders the same snapshot that the UI displays.

Reliability measures include a unified `QueryResult<T>` with warning/error severity, `CancellationToken` on every task, blocking work on `spawn_blocking`, JSON sanitization for non-finite floats, forward-compatible snapshot enums, RAII release of GPU and SetupAPI resources, and a bounded Tokio shutdown.

### 5. Platform Support

| Platform | GUI | CLI | Notes |
| --- | --- | --- | --- |
| Windows | Yes | Yes | DXGI, GDI, SetupAPI, sysinfo backends |
| Linux | Yes | Yes | X11/Wayland, sysfs, hwmon, DRM backends |
| macOS | Yes | Yes | Metal, system_profiler backends |

Field coverage differs by platform. SMART requires `smartctl`, NVIDIA temperatures require `nvidia-smi`, and some sensors require administrator/root privileges. Missing capabilities are reported as warnings rather than assumed to exist.

### 6. Quick Start

Requirements: Rust 1.95+ (edition 2024), standard C/C++ build tools, and the platform graphics development libraries. Optional: `smartctl`, `nvidia-smi`, Vulkan Loader, OpenCL ICD, OpenVX SDK.

```bash
# GUI
cargo run --release

# CLI, Markdown report
cargo run --release -- --cli --only cpu,memory,storage --out hardware.md

# JSON report
cargo run --release -- --out hardware.json

# Re-export an existing snapshot
cargo run --release -- --snapshot hardware.dvsnap --out hardware.md

# Shell completions
cargo run --release -- --completions bash
cargo run --release -- --completions zsh
cargo run --release -- --completions powershell
```

### 7. CLI Reference

```text
--cli                 Force batch/CLI mode (implied by --out/--only/--snapshot).
--out <PATH>          Write TXT, Markdown or JSON, chosen from the extension.
--only <LIST>         Comma-separated modules: cpu,memory,pcie,vulkan,opengl,
                      opencl,npu,usb,storage,motherboard,display,directx,metal,
                      openvx,system,network,audio,sensors,bandwidth.
--snapshot <PATH>     Export an existing .dvsnap/JSON snapshot instead of querying.
--quiet               Suppress progress and warning chatter.
--portable            Persist configuration and snapshots beside the executable.
--completions <SHELL> Print bash, zsh or powershell completion script.
--version             Print the version.
-h, --help            Print help.
```

The GUI starts when no batch option is supplied. Unknown modules, missing values or write failures return a non-zero exit code.

### 8. Feature Flags

```toml
default = ["vulkan", "opencl", "metal", "sensors"]
vulkan  = ["dep:ash"]
opencl  = ["dep:opencl3"]
metal   = ["dep:metal"]
openvx  = ["dep:openvx"]
sensors = ["sysinfo/component"]
```

`openvx` is off by default because it requires a local OpenVX SDK and libclang. A minimal build:

```bash
cargo run --release --no-default-features
```

### 9. Testing

```bash
cargo fmt --all -- --check
cargo check --tests
cargo test
cargo test --no-default-features
```

Coverage includes unit formatting, error severity, serialization round-trips, per-module contracts, timeout/cancellation, failure isolation, snapshot forward compatibility, non-finite sanitization, path-traversal rejection and portable root switching.

### 10. Privacy

No network requests, no telemetry, no account system. Configuration, snapshots and logs stay on the local machine; portable mode stores them beside the executable. Snapshots may contain hostnames, serial numbers, MAC addresses and IP addresses — review before sharing. See [PRIVACY.md](PRIVACY.md).

### 11. License and Commercial Licensing

This is a **custom proprietary, source-available license**, not MIT, Apache-2.0, GPL, BSD or any OSI-approved open-source license.

- Copyright in the complete project belongs exclusively to the original author.
- Individuals and non-profit organizations may use it locally for free, for learning and research.
- **All for-profit organizations, commercial teams, enterprises, studios and paid developers are prohibited** from using, copying, modifying, creating derivatives from, re-open-sourcing, re-closing, repackaging, redistributing, republishing, selling, integrating into commercial projects, offering as SaaS, shipping in paid tools or selling in courses.
- **Commercial licensing**: prior written authorization must be obtained by emailing **yanruoxinsu2@outlook.com**. Unauthorized commercial use is infringement.
- **Modification notice**: any person or organization modifying the source must open an Issue in the project repository describing the changes. Failure to report is infringement.
- **No secondary publication**: modified code may not be publicly republished, re-released, re-open-sourced or used for paid traffic.
- **Enforcement**: verified commercial infringement, unauthorized redistribution or unauthorized modified distribution will lead to a full legal process under the Copyright Law of the People's Republic of China, including civil litigation, damages, takedown of infringing projects and public accountability.
- **Interpretation and jurisdiction**: final interpretation rests with the original author; governing law is the law of mainland China.

The complete terms are in [LICENSE](LICENSE); legal boundaries and enforcement are detailed in [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md).

### 12. Disclaimer

Provided “as is”, without warranty that hardware information, SMART data, sensor readings or compatibility are complete or accurate. See [DISCLAIMER.md](DISCLAIMER.md).

### 13. Contact

- Commercial licensing: **yanruoxinsu2@outlook.com**
- Modification notice: open an Issue in the project repository with purpose, scope, affected files and behavioral changes.

---

Copyright © 2026 Device Viewer 原作者. All rights reserved.
