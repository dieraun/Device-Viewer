# 第三方组件许可声明（THIRD_PARTY_NOTICES）

本项目（Device Viewer）以 MIT OR Apache-2.0 双许可发布。发布包中链接的第三方 Rust crate 按其各自许可协议授权，著作权归各自权利人所有。

本文件依据仓库中的 `Cargo.lock` 与构建机上的 crate 源码（`~/.cargo/registry/src`）生成，列出每个组件的名称、版本、许可与上游仓库地址。本清单覆盖当前构建机可获取源码的 213 个 crate；其余 180 个为其他目标平台专用、本机尚未下载的包（见 `Cargo.lock`），发布前应在完整构建环境用 cargo-deny 复核。表中「见上游仓库」表示该 crate 以 `license-file` 字段声明许可，具体条款见其仓库内 `LICENSE` 文件。跨平台目标的完整闭包建议在任何一台构建机上重新生成并核对：

```bash
cargo install cargo-deny --locked
cargo deny check licenses
```

## 外部系统命令（不随包分发）

| 组件 | 用途 | 许可与获取方式 |
| --- | --- | --- |
| `smartctl`（smartmontools） | 读取磁盘 SMART 属性 | GPL-2.0-or-later；由操作系统包管理器或设备厂商单独提供，本项目仅调用，不随包分发 |
| `nvidia-smi`（NVIDIA 驱动） | 读取 NVIDIA GPU 信息 | NVIDIA 驱动 EULA；随显卡驱动安装，本项目仅调用，不随包分发 |

## Rust 依赖

| 组件 | 版本 | 许可 | 上游仓库 |
| --- | --- | --- | --- |
| `accesskit` | 0.25.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `accesskit_consumer` | 0.39.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `accesskit_windows` | 0.35.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `accesskit_winit` | 0.34.2 | Apache-2.0 | https://github.com/AccessKit/accesskit |
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| `arboard` | 3.7.0 | MIT OR Apache-2.0 | https://github.com/1Password/arboard |
| `arrayvec` | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| `ash` | 0.38.0+1.3.281 | MIT OR Apache-2.0 | https://github.com/ash-rs/ash |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| `bit-set` | 0.10.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set |
| `bit-vec` | 0.9.1 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec |
| `bitflags` | 0.7.0 | MIT/Apache-2.0 | https://github.com/rust-lang/bitflags |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `bytemuck_derive` | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `byteorder-lite` | 0.1.0 | 见上游仓库 | https://crates.io/crates/byteorder-lite |
| `bytes` | 1.12.1 | MIT | https://github.com/tokio-rs/bytes |
| `cc` | 1.10.0 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `cfg_aliases` | 0.2.2 | MIT | https://github.com/katharostech/cfg_aliases |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| `cl3` | 0.13.2 | Apache-2.0 | https://github.com/kenba/cl3 |
| `clipboard-win` | 5.4.3 | BSL-1.0 | https://github.com/DoumanAsh/clipboard-win |
| `codespan-reporting` | 0.13.1 | Apache-2.0 | https://github.com/brendanzab/codespan |
| `color` | 0.3.3 | Apache-2.0 OR MIT | https://github.com/linebender/color |
| `crc32fast` | 1.5.3 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `cursor-icon` | 1.2.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/cursor-icon |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | https://github.com/yaahc/displaydoc |
| `dlopen2` | 0.9.0 | MIT | https://github.com/OpenByteDev/dlopen2 |
| `dlopen2_derive` | 0.5.0 | MIT | https://github.com/OpenByteDev/dlopen2 |
| `document-features` | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |
| `ecolor` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `eframe` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/eframe |
| `egui` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `egui_glow` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui_glow |
| `egui_system_font_provider` | 0.37.0 | 见上游仓库 | https://crates.io/crates/egui_system_font_provider |
| `egui-wgpu` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-wgpu |
| `egui-winit` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-winit |
| `either` | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| `emath` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/emath |
| `epaint` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/epaint |
| `epaint_default_fonts` | 0.37.0 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | https://github.com/emilk/egui/tree/main/crates/epaint_default_fonts |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| `errno` | 0.2.8 | MIT/Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| `error-code` | 3.4.0 | BSL-1.0 | https://github.com/DoumanAsh/error-code |
| `euclid` | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| `fax` | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| `fearless_simd` | 0.4.1 | Apache-2.0 OR MIT | https://github.com/linebender/fearless_simd |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| `foldhash` | 0.1.5 | Zlib | https://github.com/orlp/foldhash |
| `foldhash` | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| `font-types` | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `fontique` | 0.11.1 | Apache-2.0 OR MIT | https://github.com/linebender/parley |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| `futures-core` | 0.3.34 | 见上游仓库 | https://crates.io/crates/futures-core |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `gl_generator` | 0.14.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `glifo` | 0.3.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `glow` | 0.17.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/grovesNL/glow |
| `glutin` | 0.32.3 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glutin_egl_sys` | 0.7.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glutin_wgl_sys` | 0.6.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `glutin-winit` | 0.5.0 | 见上游仓库 | https://crates.io/crates/glutin-winit |
| `guillotiere` | 0.7.0 | MIT/Apache-2.0 | https://github.com/nical/guillotiere |
| `half` | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| `harfrust` | 0.12.0 | MIT | https://github.com/harfbuzz/harfrust |
| `hashbrown` | 0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| `hwloc` | 0.5.0 | MIT | https://github.com/daschl/hwloc-rs |
| `icu_collections` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_locale_core` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_normalizer` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_normalizer_data` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_properties` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_properties_data` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_provider` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | https://github.com/hsivonen/idna_adapter |
| `image` | 0.25.10 | 见上游仓库 | https://crates.io/crates/image |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| `itertools` | 0.15.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| `jobserver` | 0.1.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/jobserver-rs |
| `kernel32-sys` | 0.2.2 | MIT | https://github.com/retep998/winapi-rs |
| `khronos_api` | 3.1.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `kurbo` | 0.13.1 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| `lazy_static` | 1.5.1 | MIT OR Apache-2.0 | https://github.com/rust-lang-nursery/lazy-static.rs |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| `libloading` | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| `libm` | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| `litemap` | 0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `litrs` | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `log` | 0.4.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| `memchr` | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| `memmap2` | 0.9.12 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/memmap2-rs |
| `memoffset` | 0.9.1 | MIT | https://github.com/Gilnaa/memoffset |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| `naga` | 30.0.1 | 见上游仓库 | https://crates.io/crates/naga |
| `naga-types` | 30.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `nohash-hasher` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/paritytech/nohash-hasher |
| `ntapi` | 0.4.3 | Apache-2.0 OR MIT | https://github.com/MSxDOS/ntapi |
| `num` | 0.1.43 | MIT/Apache-2.0 | https://github.com/rust-num/num |
| `num-integer` | 0.1.47 | MIT OR Apache-2.0 | https://github.com/rust-num/num-integer |
| `num-iter` | 0.1.46 | MIT OR Apache-2.0 | https://github.com/rust-num/num-iter |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| `opencl-sys` | 0.6.5 | Apache-2.0 | https://github.com/kenba/opencl-sys-rs |
| `opencl3` | 0.12.4 | 见上游仓库 | https://crates.io/crates/opencl3 |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `parlance` | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/parley |
| `pci-info` | 0.4.0 | MIT OR Apache-2.0 | https://github.com/xanathar/pci-info |
| `peniko` | 0.6.1 | Apache-2.0 OR MIT | https://github.com/linebender/peniko |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/pkg-config-rs |
| `png` | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| `poll-promise` | 0.3.0 | MIT OR Apache-2.0 | https://github.com/EmbarkStudios/poll-promise |
| `polycool` | 0.4.0 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| `potential_utf` | 0.1.6 | 见上游仓库 | https://crates.io/crates/potential_utf |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| `profiling` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| `quick-error` | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| `read-fonts` | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `regex-syntax` | 0.8.11 | 见上游仓库 | https://crates.io/crates/regex-syntax |
| `renderdoc-sys` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/ebkalderon/renderdoc-rs |
| `ring` | 0.17.14 | 见上游仓库 | https://crates.io/crates/ring |
| `rustc-hash` | 1.1.0 | Apache-2.0/MIT | https://github.com/rust-lang-nursery/rustc-hash |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| `rustls` | 0.23.46 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls |
| `rustls-native-certs` | 0.7.3 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls-native-certs |
| `rustls-pemfile` | 2.2.0 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/pemfile |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | https://github.com/rustls/pki-types |
| `rustls-webpki` | 0.103.16 | ISC | https://github.com/rustls/webpki |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion |
| `same-file` | 1.0.6 | Unlicense/MIT | https://github.com/BurntSushi/same-file |
| `schannel` | 0.1.30 | MIT | https://github.com/steffengy/schannel-rs |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex |
| `simd-adler32` | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| `simdutf8` | 0.1.5 | MIT OR Apache-2.0 | https://github.com/rusticstuff/simdutf8 |
| `siphasher` | 1.0.4 | 见上游仓库 | https://crates.io/crates/siphasher |
| `skrifa` | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `slab` | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| `smallvec` | 1.16.4 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| `smartctl-rs` | 0.2.0 | MIT | https://github.com/einstein8612/smartctl-rs.git |
| `smol_str` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | https://github.com/storyyeller/stable_deref_trait |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| `subtle` | 2.6.1 | BSD-3-Clause | https://github.com/dalek-cryptography/subtle |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `syn` | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `synstructure` | 0.14.0 | MIT | https://github.com/mystor/synstructure |
| `sys-locale` | 0.3.2 | MIT OR Apache-2.0 | https://github.com/1Password/sys-locale |
| `sysinfo` | 0.40.0 | MIT | https://github.com/GuillaumeGomez/sysinfo |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `tiff` | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| `tinystr` | 0.8.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `tokio` | 1.54.1 | MIT | https://github.com/tokio-rs/tokio |
| `tokio-macros` | 2.7.2 | MIT | https://github.com/tokio-rs/tokio |
| `tokio-util` | 0.7.20 | MIT | https://github.com/tokio-rs/tokio |
| `tracing` | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-attributes` | 0.1.31 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-core` | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| `type-map` | 0.5.1 | MIT/Apache-2.0 | https://github.com/kardeiz/type-map |
| `unicode-general-category` | 1.1.0 | Apache-2.0 | https://github.com/yeslogic/unicode-general-category |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| `unicode-properties` | 0.1.4 | MIT/Apache-2.0 | https://github.com/unicode-rs/unicode-properties |
| `unicode-script` | 0.5.8 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-script |
| `unicode-segmentation` | 1.13.3 | 见上游仓库 | https://crates.io/crates/unicode-segmentation |
| `unicode-width` | 0.2.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-width |
| `untrusted` | 0.9.0 | ISC | https://github.com/briansmith/untrusted |
| `ureq` | 2.12.1 | MIT OR Apache-2.0 | https://github.com/algesten/ureq |
| `url` | 2.5.8 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| `utf_types` | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf_types |
| `utf8_iter` | 1.1.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf8_iter |
| `uuid` | 1.29.0 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| `vello_common` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `vello_cpu` | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | https://github.com/SergioBenitez/version_check |
| `walkdir` | 2.5.0 | 见上游仓库 | https://crates.io/crates/walkdir |

---

本文件为合规证据链的一部分，随仓库更新；如发现遗漏或错误，请提交 Issue。
