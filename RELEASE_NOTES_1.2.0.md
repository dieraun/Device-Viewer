# Device Viewer 1.2.0

Vulkan 状态语义与深色界面更新 · Clearer Vulkan status and refined dark UI

2026-10-31

---

## 中文

Device Viewer 1.2.0 是 1.1.0 之后的界面与发布可靠性版本。本版本不改变硬件采集架构、快照格式、CLI 参数或页面结构，重点修正 Vulkan 状态的误导性文案，统一深色界面的视觉层级，并修复 GitHub Actions 中 Windows 构建取消会连带取消 Release 发布的问题。

### 新增与改进

- **Vulkan 状态文案修正**：概览页与 Vulkan 页不再用“不适用”描述“当前环境没有可用的 Vulkan 后端”。当 Vulkan 采集缺失、加载器不可用或没有枚举到设备时，统一显示“无Vulkan支持”，并保留 GPU 名称作为辅助信息。
- **准确的悬浮说明**：GPU 卡片与 Vulkan 页的状态徽标提供说明：“该GPU不兼容Vulkan，不影响基础硬件读取。”明确这是后端能力提示，不是硬件故障，也不影响 CPU、内存、存储、网络等基础采集。
- **深色界面视觉统一**：页面、卡片、浮层使用三层独立背景；卡片改为更柔和的圆角、细边框与低透明度阴影；侧边栏和顶部工具栏使用更轻的背景与分隔线，减少大面积纯黑带来的沉闷感。
- **信息层级与可读性**：主文字、次级文字、弱化文字分级；指标数值字号加大；进度条改为圆角、强调色填充与独立轨道；信息、警告、错误、成功徽标使用不同颜色，能力清单中的失败项按严重程度着色。
- **浅色主题同步**：深色与浅色主题共用新的间距、字号、圆角与控件状态，不再只对深色主题做局部覆盖。
- **发布流程可靠性**：Windows x86_64 构建任务增加 120 分钟超时与 `continue-on-error`，保留 Cargo 构建缓存；Publish 任务在 macOS 或 Linux 至少一个平台成功时即可发布，Windows 失败或取消不再导致整个 Release 被跳过。
- **缺失产物处理**：发布任务只上传实际存在的安装包；如果 Windows、macOS 或 Linux 包缺失，会在 Release notes 中自动追加中英文“缺失的安装包”说明，而不是让发布流程静默失败。
- **版本与构建信息**：版本号更新为 1.2.0；关于页面继续显示构建目标、Profile、Git Commit、构建时间与 rustc 版本。

### 下载（Windows）

| 平台 | 架构 | 文件 |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.2.0.zip` |

本版本仅提供 Windows x86_64 预编译包；macOS 与 Linux 可从同一份源码构建，本 Release 不附带这两个平台的预编译二进制。

压缩包包含 `DeviceViewer.exe`、`README.md`、`LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`、平台说明与发行说明，并附带 SHA-256 校验文件。

### 升级说明

- 直接替换旧版 `DeviceViewer.exe`，或使用 1.2.0 压缩包完整覆盖。
- 配置文件字段保持兼容；语言、主题、更新检查、单位制、便携模式、窗口位置、筛选条件与收藏扩展会继续读取。
- `.dvsnap` 快照格式与 schema 版本未变化，1.0.x / 1.1.x 的快照仍可读取和对比。
- CLI 参数、导出格式和环境变量未变化。
- 便携模式仍将配置、日志与快照保存在可执行文件旁边。

### 运行要求

- 64 位 Windows 10/11 或 Windows Server 2016 及以上版本。
- Microsoft Visual C++ 2015–2022 运行库（x64）。
- 读取 SMART、部分传感器或受限设备信息时建议以管理员身份运行；权限不足会显示为非阻塞警告。
- 更新检查需要能够访问 `api.github.com`；离线环境会显示为提示，不影响其他功能。
- OpenVX 为可选编译特性，官方 Windows 二进制未启用；面板会明确显示“未启用”。

### 已知限制

- 硬件、固件、驱动与操作系统版本差异会影响可用字段；缺失数据会显示为不可用，而不是猜测值。
- 虚拟机、远程桌面和部分服务器平台可能不暴露温度、风扇或 GPU 遥测。
- “无Vulkan支持”表示当前环境未提供可用的 Vulkan 后端，不表示 GPU 存在硬件故障；安装或修复 Vulkan 运行库后重新启动即可重新检测。
- GitHub API 有匿名请求频率限制；频繁检查可能暂时失败，稍后重试即可。
- 内存带宽为只读基准测试，不代表网络带宽或磁盘吞吐。
- Windows 构建在 GitHub 托管 Runner 上仍可能因平台侧取消而缺失安装包；此时 Release 会保留其他平台产物并注明缺失项。

### 发布与构建说明

- 官方 Windows 包使用 `cargo build --release --locked --no-default-features --features "vulkan,opencl,sensors"` 构建。
- 发布工作流支持标签推送 `v*` 与手动指定标签；手动触发时以实际 checkout 的提交为准。
- Windows 任务的 `continue-on-error` 只用于避免单个平台阻断整个 Release，不代表构建失败可以忽略；发布后仍需检查产物与缺失说明。

### 隐私说明

更新检查只向 GitHub Releases API 发起只读 HTTPS 请求，不携带硬件信息、快照、配置内容、用户标识或遥测数据。应用不会上传任何本地采集结果；关闭自动检查后不会在启动时发起网络请求。

### 许可与商业使用

本项目为**源码可见的专有软件**，不是 OSI 开源项目。个人与非营利组织可免费本地使用、学习研究。任何公司、商业团队、工作室或付费开发者，未经作者书面授权，不得使用、复制、修改、衍生、二次开源、二次闭源、打包、分发、发布、售卖、集成进商业项目、SaaS、付费工具或课程。商业授权请联系：**yanruoxinsu2@outlook.com**。

修改源代码必须在项目仓库提交 Modify Source Issue 报备；未报备视为侵权。GitHub Fork 仅是平台功能，不构成授权，禁止 Fork 后二次发布、分发或商用。

---

## English

Device Viewer 1.2.0 is a UI and release-reliability release following 1.1.0. It does not change the hardware collection architecture, snapshot format, CLI arguments, or page structure. The release corrects misleading Vulkan status text, applies a consistent dark visual hierarchy, and fixes the GitHub Actions behavior where a cancelled Windows build also cancelled the Release publication.

### Added and improved

- **Clearer Vulkan status**: the Overview and Vulkan pages no longer use “N/A” for “no usable Vulkan backend in this environment”. When Vulkan collection is unavailable, the loader is missing, or no devices are enumerated, the UI shows “No Vulkan support” and keeps the GPU name as supporting context.
- **Accurate hover explanation**: the GPU card and Vulkan status badge explain: “This GPU is not Vulkan-compatible; basic hardware inspection is unaffected.” This is a backend capability notice, not a hardware fault, and does not affect CPU, memory, storage, network, or other collection.
- **Unified dark interface**: page, card, and overlay layers use distinct backgrounds; cards use softer corners, thin borders, and low-opacity shadows; the sidebar and top toolbar use lighter surfaces and separators instead of large areas of flat black.
- **Readable hierarchy**: primary, secondary, and muted text are separated; metric values are larger; progress bars have rounded accents and a distinct track; info, warning, error, and success badges use different colors, and checklist failures are colored by severity.
- **Light theme parity**: dark and light themes share the new spacing, type scale, corner radii, and widget states instead of styling dark mode only.
- **Release workflow reliability**: the Windows x86_64 job now has a 120-minute timeout and `continue-on-error`, while retaining the Cargo build cache. Publish runs when macOS or Linux succeeds, so a failed or cancelled Windows job no longer blocks the entire Release.
- **Missing-artifact handling**: the publish job uploads only artifacts that exist. If the Windows, macOS, or Linux package is missing, the Release notes automatically include a bilingual “Missing packages” section instead of failing silently.
- **Version and build information**: the version is now 1.2.0; the About page continues to show the build target, profile, Git commit, build time, and rustc version.

### Downloads (Windows)

| Platform | Architecture | File |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.2.0.zip` |

This release provides a Windows x86_64 prebuilt package only. macOS and Linux can be built from the same source; no prebuilt binaries for those platforms are attached to this release.

The archive contains `DeviceViewer.exe`, `README.md`, `LICENSE`, `LEGAL_STATEMENT.md`, `PRIVACY.md`, `DISCLAIMER.md`, platform notes, and these release notes, plus a SHA-256 checksum file.

### Upgrade notes

- Replace the old `DeviceViewer.exe`, or overwrite the installation with the 1.2.0 archive.
- Configuration fields remain compatible; language, theme, update checks, units, portable mode, window position, filters, and favorite extensions are preserved.
- The `.dvsnap` snapshot format and schema version are unchanged; 1.0.x and 1.1.x snapshots remain readable and comparable.
- CLI arguments, export formats, and environment variables are unchanged.
- Portable mode still keeps configuration, logs, and snapshots beside the executable.

### Requirements

- 64-bit Windows 10/11 or Windows Server 2016 and newer.
- Microsoft Visual C++ 2015–2022 runtime (x64).
- Administrator rights are recommended for SMART, some sensors, and restricted devices. Permission problems appear as non-blocking warnings.
- Update checks require access to `api.github.com`; offline environments show a message without affecting other features.
- **OpenVX** is an optional compile-time feature and is not enabled in the official Windows binary. The panel reports it as unavailable.

### Known limitations

- Hardware, firmware, driver, and OS differences affect which fields are available. Missing data is shown as unavailable rather than guessed.
- Virtual machines, remote desktop sessions, and some server platforms may not expose temperature, fan, or GPU telemetry.
- “No Vulkan support” means no usable Vulkan backend is available in the current environment; it does not indicate a GPU hardware fault. Installing or repairing the Vulkan runtime and restarting the application triggers a fresh detection.
- The GitHub API has anonymous rate limits; frequent checks may fail temporarily and can be retried later.
- The memory bandwidth figure is a read-only benchmark, not network throughput or disk I/O.
- The Windows build on GitHub-hosted runners can still be cancelled by the platform and leave no package; the release keeps other platform assets and lists the missing package.

### Release and build notes

- The official Windows package is built with `cargo build --release --locked --no-default-features --features "vulkan,opencl,sensors"`.
- The workflow supports tag pushes matching `v*` and manual dispatch with an explicit tag; manual runs use the checked-out commit.
- `continue-on-error` on the Windows job exists only to keep one platform from blocking the whole release; it does not make a failed build acceptable. Always verify published assets and the missing-package note.

### Privacy

Update checks make a read-only HTTPS request to the GitHub Releases API. They do not send hardware information, snapshots, configuration contents, user identifiers, or telemetry. The application never uploads collection results. With automatic checks disabled, no network request is made at startup.

### License and commercial use

This is **source-visible proprietary software**, not an OSI open-source project. Individuals and non-profit organizations may use it locally for free and study the source. Companies, commercial teams, studios, and paid developers may not use, copy, modify, create derivatives from, re-open-source, re-close, package, distribute, publish, sell, integrate into commercial projects, offer as SaaS, or include in paid tools or courses without prior written permission from the author. Commercial licensing: **yanruoxinsu2@outlook.com**.

Any modification must be reported in a Modify Source Issue in the project repository; failure to report is infringement. A GitHub Fork is a platform feature, not an authorization; do not publish, distribute, or commercially use a fork.
