# Device Viewer 1.1.0

系统跟随、关于页面与更新检查 · System theme, About page and update checks

2026-10-30

---

## 中文

Device Viewer 1.1.0 是 1.0.1 之后的功能版本，重点补齐桌面应用应有的系统集成能力：语言与主题跟随操作系统、完整的关于页面、GitHub Release 更新检查，以及侧边栏导航布局修复。硬件采集架构、快照格式和 CLI 用法保持不变。

### 新增与改进

- **语言跟随系统**：语言选项新增“跟随系统”，首次启动默认使用系统语言。Windows 读取系统 UI 语言，macOS 读取 `AppleLanguages`，其他平台回退到 `LC_ALL`、`LC_MESSAGES` 与 `LANG`。切换到“跟随系统”后，应用会按当前系统语言解析中英文；旧配置中的 `english`、`chinese` 仍可直接读取。
- **主题跟随系统**：主题选项新增“跟随系统”，首次启动默认跟随系统明暗模式。使用 egui 的系统主题偏好实现，系统主题变化时会随界面重绘生效；也可手动固定为深色或浅色。旧配置中的 `dark: true/false` 会自动迁移到新的主题字段。
- **关于页面**：新增 About 页面，展示作者 `dieraun`、GitHub 主页、应用版本、Rust 编译器版本、构建目标、构建 Profile、Git Commit、构建时间、运行时操作系统、架构、平台族、指针宽度，以及许可证、法律声明、隐私声明与免责声明入口。
- **GitHub 更新检查**：新增 Updates 页面，通过 GitHub Releases API 检查新版本；支持“包含预发布版本”，首次启动默认自动检查。页面展示当前版本、最新版本、标签、名称、发布时间、预发布状态、是否可更新，并提供 Release 链接。网络不可达、仓库无 Release、GitHub 限流、JSON 解析失败等情况都会显示为非阻塞提示，不影响硬件采集和主界面。
- **构建信息**：构建脚本会把目标平台、构建 Profile、版本号、Git Commit、构建时间与 rustc 版本写入二进制，关于页面直接读取，不依赖运行时外部命令。
- **侧边栏修复**：导航列表改为限高滚动区域，版本号与警告统计固定在底部，不再遮挡 About、Updates、Settings 等末尾入口。
- **配置持久化**：语言、主题、自动检查更新、包含预发布版本、单位制、便携模式、窗口位置、筛选条件与收藏扩展均写入配置文件，重启后恢复。
- **依赖更新**：新增 `ureq` 2.12（启用平台原生证书）用于 HTTPS 更新检查；Windows 目标增加 `Win32_Globalization` 特性用于读取系统 UI 语言。

### 下载（Windows）

| 平台 | 架构 | 文件 |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.1.0.zip` |

本版本仅提供 Windows x86_64 预编译包；macOS 与 Linux 可从同一份源码构建，本 Release 不附带这两个平台的预编译二进制。

压缩包包含 `DeviceViewer.exe`、`README.md`、`LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`、平台说明与发行说明，并附带 SHA-256 校验文件。

### 升级说明

- 直接替换旧版 `DeviceViewer.exe`，或使用 1.1.0 压缩包完整覆盖。
- 配置字段会自动迁移：旧 `language` 的 `english` / `chinese` 继续有效，新值还包括 `system`；旧 `dark` 布尔值会转换为 `theme` 的 `dark` / `light`，随后不再写出旧字段。
- `.dvsnap` 快照格式与 schema 版本未变化，1.0.x 的快照仍可读取和对比。
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
- 系统语言与系统主题的识别依赖桌面环境；极简窗口管理器或非标准环境可能回退到英文/深色。
- GitHub API 有匿名请求频率限制；频繁检查可能暂时失败，稍后重试即可。
- 内存带宽为只读基准测试，不代表网络带宽或磁盘吞吐。

### 隐私说明

更新检查只向 GitHub Releases API 发起只读 HTTPS 请求，不携带硬件信息、快照、配置内容、用户标识或遥测数据。应用不会上传任何本地采集结果；关闭自动检查后不会在启动时发起网络请求。

### 许可与商业使用

本项目为**源码可见的专有软件**，不是 OSI 开源项目。个人与非营利组织可免费本地使用、学习研究。任何公司、商业团队、工作室或付费开发者，未经作者书面授权，不得使用、复制、修改、衍生、二次开源、二次闭源、打包、分发、发布、售卖、集成进商业项目、SaaS、付费工具或课程。商业授权请联系：**yanruoxinsu2@outlook.com**。

修改源代码必须在项目仓库提交 Modify Source Issue 报备；未报备视为侵权。GitHub Fork 仅是平台功能，不构成授权，禁止 Fork 后二次发布、分发或商用。

---

## English

Device Viewer 1.1.0 is a feature release following 1.0.1. It adds the desktop integration expected from a native application: system-following language and theme preferences, a complete About page, GitHub Release update checks, and a sidebar navigation layout fix. The hardware collection architecture, snapshot format, and CLI behavior are unchanged.

### Added and improved

- **Follow system language**: the language selector now includes “Follow system”, which is the first-launch default. Windows reads the system UI language, macOS reads `AppleLanguages`, and other platforms fall back to `LC_ALL`, `LC_MESSAGES`, and `LANG`. English and Chinese resolve from the current system language; legacy `english` and `chinese` configuration values remain readable.
- **Follow system theme**: the theme selector now includes “Follow system”, the first-launch default. It uses egui's system theme preference, so OS light/dark changes apply on repaint; dark and light can still be pinned manually. Legacy `dark: true/false` values migrate automatically.
- **About page**: shows author `dieraun`, GitHub profile, application version, Rust compiler version, build target, build profile, Git commit, build time, runtime OS, architecture, platform family, pointer width, and links to the license, legal statement, privacy statement, and disclaimer.
- **GitHub update checks**: the Updates page queries the GitHub Releases API, supports an “include pre-releases” option, and performs an automatic check on first launch by default. It displays the current and latest versions, tag, release name, publish time, pre-release state, availability, and a Release link. Network failures, missing releases, GitHub rate limits, and JSON parse errors are shown as non-blocking messages and never interrupt hardware collection or the main UI.
- **Build metadata**: the build script embeds target, profile, version, Git commit, build time, and rustc version into the binary; the About page reads them directly without invoking external commands at runtime.
- **Sidebar fix**: navigation is now a height-bounded scroll area, with version and warning counts fixed at the bottom, so About, Updates, and Settings are no longer covered by the footer.
- **Persisted settings**: language, theme, automatic update checks, pre-release checks, units, portable mode, window position, filters, and favorite extensions are saved and restored across restarts.
- **Dependency updates**: added `ureq` 2.12 with platform-native certificates for HTTPS update checks, and enabled the `Win32_Globalization` Windows feature for reading the system UI language.

### Downloads (Windows)

| Platform | Architecture | File |
| --- | --- | --- |
| Windows | x86_64 | `DeviceViewer-windows-x86_64-1.1.0.zip` |

This release provides a Windows x86_64 prebuilt package only. macOS and Linux can be built from the same source; no prebuilt binaries for those platforms are attached to this release.

The archive contains `DeviceViewer.exe`, `README.md`, `LICENSE`, `LEGAL_STATEMENT.md`, `PRIVACY.md`, `DISCLAIMER.md`, platform notes, and these release notes, plus a SHA-256 checksum file.

### Upgrade notes

- Replace the old `DeviceViewer.exe`, or overwrite the installation with the 1.1.0 archive.
- Configuration migrates automatically: legacy `language` values `english` and `chinese` remain valid, with `system` added; legacy `dark` booleans map to `theme` values `dark` and `light`, and the old field is no longer written.
- The `.dvsnap` snapshot format and schema version are unchanged; 1.0.x snapshots remain readable and comparable.
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
- System language and theme detection depend on the desktop environment; minimal window managers or unusual setups may fall back to English/dark.
- The GitHub API has anonymous rate limits; frequent checks may fail temporarily and can be retried later.
- The memory bandwidth figure is a read-only benchmark, not network throughput or disk I/O.

### Privacy

Update checks make a read-only HTTPS request to the GitHub Releases API. They do not send hardware information, snapshots, configuration contents, user identifiers, or telemetry. The application never uploads collection results. With automatic checks disabled, no network request is made at startup.

### License and commercial use

This is **source-visible proprietary software**, not an OSI open-source project. Individuals and non-profit organizations may use it locally for free and study the source. Companies, commercial teams, studios, and paid developers may not use, copy, modify, create derivatives from, re-open-source, re-close, package, distribute, publish, sell, integrate into commercial projects, offer as SaaS, or include in paid tools or courses without prior written permission from the author. Commercial licensing: **yanruoxinsu2@outlook.com**.

Any modification must be reported in a Modify Source Issue in the project repository; failure to report is infringement. A GitHub Fork is a platform feature, not an authorization; do not publish, distribute, or commercially use a fork.
