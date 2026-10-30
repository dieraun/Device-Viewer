# Device Viewer 打包与运行说明

本文档随每个发布包一起分发，说明各平台运行前置依赖、权限边界、便携模式、构建 feature 与校验方式。

## 1. 产物命名

发布包统一命名为：

```text
DeviceViewer-<platform>-<arch>-<version>
```

当前发布矩阵：

| 平台 | 架构 | 目标三元组 | 归档格式 |
| --- | --- | --- | --- |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `.zip` |
| macOS | x86_64 | `x86_64-apple-darwin` | `.zip` |
| macOS | aarch64 | `aarch64-apple-darwin` | `.zip` |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `.tar.gz` |

每个归档都包含：

- 可执行文件 `DeviceViewer`（Windows 为 `DeviceViewer.exe`）
- `LICENSE`
- `LEGAL_STATEMENT.md`
- `PRIVACY.md`
- `DISCLAIMER.md`
- 本说明 `README_PACKAGE.md`
- 对应的 `.sha256` 校验文件（归档外）

## 2. Windows x86_64

### 2.1 构建前置依赖

- Windows 10/11 x64。
- **Microsoft Visual Studio 2022 Build Tools**，安装工作负载 **Desktop development with C++**，至少包含：
  - MSVC v143 x64/x86 build tools
  - Windows 10/11 SDK
- Rust stable，目标 `x86_64-pc-windows-msvc`。
- 设置好 `%CARGO_HOME%` 与 `%CARGO_TARGET_DIR%`（可选）。

> 仅运行发布包不需要安装 Rust 或 Visual Studio；只有从源码构建才需要 MSVC C++ Build Tools。

### 2.2 运行前置依赖

- **Microsoft Visual C++ 2015-2022 Redistributable (x64)**。Rust MSVC 默认使用动态 CRT，缺少运行库时会出现 `VCRUNTIME140.dll` / `MSVCP140.dll` 缺失。
- 显示与 GPU 面板：
  - OpenGL 3.3 以上的显卡驱动（eframe/egui 的默认 glow 后端）。
  - Vulkan 面板需要系统安装 Vulkan Loader（`vulkan-1.dll`）和厂商 ICD；未安装时该面板显示为不可用，不影响其他页面。
  - OpenCL 面板需要厂商 OpenCL ICD；未安装时显示为不可用。
  - DirectX 面板使用系统 DXGI，不需要额外运行库。
- 存储 SMART 面板：
  - 程序不捆绑 `smartctl`。需要 SMART 时，请自行安装 `smartctl.exe` 并加入 `PATH`，或放在程序可访问目录。
  - 没有 `smartctl` 时，程序仍会通过 sysinfo 显示磁盘容量、文件系统、挂载点等基础信息，SMART 字段为空。
- 传感器面板：
  - 部分温度、风扇、电压来自主板/EC，需要管理员权限或厂商驱动；权限不足时对应项缺失并给出警告。
  - NVIDIA GPU 温度可选调用 `nvidia-smi`；没有该命令时跳过。

### 2.3 权限说明

- 普通查看 CPU、内存、PCIe、USB、显示、网络、音频、DirectX 不需要管理员权限。
- SMART、部分传感器、部分 USB 设备描述符在 Windows 上可能需要**以管理员身份运行**。
- 程序不会主动提权，也不会修改系统设置；权限不足只影响对应采集项。

### 2.4 便携模式

```powershell
DeviceViewer.exe --portable
```

便携模式下，配置、日志和快照保存在可执行文件所在目录，而不是 `%APPDATA%\DeviceViewer`。不要把程序放在 `C:\Program Files` 等普通用户不可写的目录；如必须如此，请改用默认为非便携模式，或把整个目录复制到用户可写位置。

## 3. macOS x86_64 / aarch64

### 3.1 构建前置依赖

- macOS 11 或更高版本。
- Xcode Command Line Tools（`xcode-select --install`）。
- Rust stable；按架构安装目标：
  - Intel：`rustup target add x86_64-apple-darwin`
  - Apple Silicon：`rustup target add aarch64-apple-darwin`

### 3.2 运行前置依赖

- 系统自带 Metal、OpenGL、OpenCL 框架。
- Vulkan 面板需要额外安装 MoltenVK / Vulkan Loader；未安装时该面板显示为不可用。
- OpenCL 在新版 macOS 上已被标记为弃用，但仍可能可用；不可用时只影响 OpenCL 面板。
- SMART 需要自行安装 `smartctl`（例如通过 Homebrew）并以 `sudo` 运行；未安装时仅显示基础磁盘信息。

### 3.3 签名与公证提示

发布包默认**未签名、未公证**。仓库和 CI 工作流不包含任何 Apple 证书、私钥、App-Specific Password 或 API Key。首次打开可能被 Gatekeeper 拦截，请在“系统设置 → 隐私与安全性”中允许，或在确认 SHA-256 后执行：

```bash
xattr -dr com.apple.quarantine DeviceViewer
```

正式签名/公证由发布维护者使用自己的 Apple Developer ID 完成，命令示例见包内 `SIGNING_AND_NOTARIZATION.txt`。签名材料不得提交到仓库。

### 3.4 权限说明

- 常规硬件信息不需要 root。
- `smartctl`、部分传感器和某些 USB/PCIe 描述符可能需要 `sudo`。
- 程序本身不安装守护进程、不修改系统扩展。

### 3.5 便携模式

```bash
./DeviceViewer --portable
```

macOS 的便携配置、日志和快照同样位于可执行文件旁。若把程序放入 `/Applications`，请使用默认配置目录，避免写入失败。

## 4. Linux x86_64 (musl)

### 4.1 构建前置依赖

CI 在 Ubuntu 22.04 上构建 `x86_64-unknown-linux-musl`，安装：

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  musl-tools pkg-config \
  libx11-dev libx11-xcb-dev libxcb1-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev \
  libgl1-mesa-dev libegl1-mesa-dev libfontconfig1-dev
rustup target add x86_64-unknown-linux-musl
```

构建命令：

```bash
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"
cargo build --release --target x86_64-unknown-linux-musl \
  --no-default-features --features "vulkan,opencl,sensors"
```

### 4.2 静态链接说明

发布包使用 musl 目标并启用 `crt-static`。Rust 标准库与 Rust 依赖静态链接；X11、Wayland、OpenGL/EGL 等桌面集成库由系统在运行时提供（部分通过 `dlopen` 加载）。因此：

- 不依赖构建机的 glibc 版本。
- 仍需要目标桌面环境具备 X11 或 Wayland、Mesa/厂商 GL 驱动。
- 无图形环境时可使用 `--cli`，不需要 X11/Wayland。

### 4.3 运行前置依赖

- 64 位 Linux，glibc 或 musl 桌面环境均可；发布包自身使用 musl。
- X11 或 Wayland 会话，以及 `libGL` / `libEGL`（通常由 Mesa 或显卡驱动提供）。
- `libxkbcommon`、`libwayland-client`、`libX11`（桌面环境通常已安装）。
- Vulkan 面板需要 `libvulkan.so.1` 与厂商 ICD；OpenCL 面板需要 ICD。
- SMART 需要安装 `smartctl` 并以 root 运行；传感器可能读取 `/sys/class/hwmon`，部分项目需要 root。
- 可选 `nvidia-smi` 用于 NVIDIA 温度。

### 4.4 权限说明

- 普通硬件信息不需要 root。
- SMART、部分 hwmon 传感器、DMI 序列号等需要 root 或相应 udev 权限。
- 程序只读采集，不写入硬件寄存器，不修改内核模块。

### 4.5 便携模式

```bash
./DeviceViewer --portable
```

配置、日志、快照位于可执行文件旁。若安装在 `/usr/bin` 或只读挂载点，请使用默认配置目录（`$XDG_CONFIG_HOME/device-viewer` 或 `~/.config/device-viewer`）。

## 5. 通用使用说明

### 5.1 GUI

直接运行可执行文件。首次启动会显示采集进度页；采集按模块并行执行，单个模块失败不会中断其他模块。

### 5.2 CLI

```bash
DeviceViewer --cli --out report.json
DeviceViewer --cli --only cpu,memory,pcie --out report.md
DeviceViewer --cli --snapshot report.json --out report.txt
DeviceViewer --cli --quiet --out report.json
DeviceViewer --completions bash
```

- `--out` 根据扩展名推断 TXT / Markdown / JSON。
- `--only` 接受逗号分隔的模块名；未知模块会直接报错退出。
- `--snapshot` 读取已有快照并重新导出，不重新采集。
- `--quiet` 抑制非错误输出，适合脚本。
- `--portable` 切换配置与快照目录。
- `--completions bash|zsh|powershell` 输出补全脚本。

### 5.3 快照与对比

快照默认保存为 `.dvsnap`（JSON 内容），包含 schema 版本与采集时间。快照对比会标记新增、删除、变更字段；未知字段和未知枚举值在前向兼容模式下会被安全忽略。

### 5.4 配置与日志位置

| 模式 | Windows | macOS | Linux |
| --- | --- | --- | --- |
| 默认配置 | `%APPDATA%\DeviceViewer` | `~/Library/Application Support/DeviceViewer` | `$XDG_CONFIG_HOME/device-viewer` 或 `~/.config/device-viewer` |
| 便携配置 | 可执行文件旁 | 可执行文件旁 | 可执行文件旁 |

日志文件名为 `device-viewer.log`，快照目录为 `snapshots/`。

## 6. Feature 与裁剪

默认 feature：

```toml
default = ["vulkan", "opencl", "metal", "sensors"]
```

| Feature | 作用 | 说明 |
| --- | --- | --- |
| `vulkan` | Vulkan 设备、显存、扩展采集 | 需要 `ash`，运行时需要 Vulkan Loader |
| `opencl` | OpenCL 平台/设备采集 | 动态加载 ICD，未安装时只影响该面板 |
| `metal` | macOS Metal 设备采集 | 仅 macOS 有效 |
| `sensors` | 温度、风扇、电压 | 需要 sysinfo `component` |
| `openvx` | OpenVX 可用性探测 | 默认关闭；需要原生 OpenVX SDK 与 libclang |

裁剪示例：

```bash
# 最小采集，无 GPU 运行时
cargo build --release --no-default-features --features "sensors"

# 仅 Vulkan + 传感器
cargo build --release --no-default-features --features "vulkan,sensors"
```

关闭某 feature 后，对应面板仍会显示“不可用”警告，不会导致程序崩溃。

## 7. 校验与解压

每个归档旁提供 `.sha256` 文件。

Windows：

```powershell
Get-FileHash .\DeviceViewer-windows-x86_64-<version>.zip -Algorithm SHA256
```

macOS：

```bash
shasum -a 256 DeviceViewer-macos-<arch>-<version>.zip
```

Linux：

```bash
sha256sum -c DeviceViewer-linux-x86_64-<version>.tar.gz.sha256
tar -xzf DeviceViewer-linux-x86_64-<version>.tar.gz
```

## 8. 法律与隐私

本项目采用定制化专有许可，**不是** MIT、Apache-2.0、GPL、BSD 或任何 OSI 认可的开源许可证。

- 个人、非营利组织可免费本地使用、学习研究。
- 任何盈利组织、商业团队、企业、工作室、付费开发者，未经作者书面授权，禁止使用、复制、修改、衍生、二次开源、二次闭源、二次打包、二次分发、二次发布、商用售卖、集成进商业项目、SaaS、付费工具、课程售卖。
- 商业授权请联系：**yanruoxinsu2@outlook.com**。
- 修改源代码必须在项目仓库提交 Issue 报备；禁止修改后二次公开、二次发布、二次开源、二次引流售卖。
- 完整条款见包内 `LICENSE`、`LEGAL_STATEMENT.md`、`PRIVACY.md`、`DISCLAIMER.md`。

程序纯本地运行，不上传硬件信息、快照、日志或使用统计。分享快照前请自行检查其中可能包含的主机名、序列号、MAC 地址、IP 地址等设备标识。
