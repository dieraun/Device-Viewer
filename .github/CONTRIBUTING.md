# 贡献指南 / Contributing to Device Viewer

感谢你愿意为 Device Viewer 做出贡献。Issue、文档修订、缺陷修复、新采集模块、界面改进都欢迎。

本项目采用 **MIT OR Apache-2.0 双许可**，是 OSI 认可的开源软件。

---

## 行为准则

参与者须遵守 [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/)：

- 使用友好、包容的语言，尊重不同经验与背景的贡献者。
- 接受建设性批评，就事论事，不针对个人。
- 不发布侮辱、歧视、骚扰、泄露隐私或与项目无关的挑衅内容。
- 维护者有权删除不当内容，并对持续违规者采取警告、临时或永久封禁措施。

---

## 开发环境

- **Rust 1.100+**（以 `Cargo.toml` 的 `rust-version` 为准），稳定版工具链。
- 平台：Windows / Linux / macOS 任一即可，跨平台改动请说明测试平台。
- 可选系统依赖：`smartctl`（smartmontools）、`nvidia-smi`（NVIDIA 驱动）；缺失时相关模块会降级，不影响其他模块。

提交前请在本地跑通：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
```

---

## 提交流程

1. **先看 Issue**：缺陷或功能建议请先搜索现有 Issue，避免重复。
2. **Fork 并建分支**：从 `main` 拉出分支，命名如 `fix/snapshot-timeout`、`feat/hwloc-cache`。
3. **小步提交**：一个 PR 只做一件事；无关的重构、格式化另开 PR。
4. **写清 PR 描述**：改动目的、影响范围、验证方式（命令与结果）、涉及的平台。
5. **CI 通过**：`fmt`、`clippy`、`test` 全绿后才会进入 review。
6. **Review 与合并**：至少一名维护者批准后合并；维护者可能要求拆分或补充测试。

---

## 代码规范

- `cargo fmt` 必须通过，不提交未格式化的代码。
- `cargo clippy --all-targets -- -D warnings` 零警告。
- 新增采集模块须带单元测试；修复缺陷须带能复现该缺陷的测试。
- 采集层只产出可序列化的结构化数据，界面层只读取数据，不直接调用系统接口。
- 单个采集模块失败不得中断其他模块；超时、权限不足、设备缺失都按降级处理。
- 不新增网络请求与遥测；确有必要时在 PR 中单独说明并同步更新 [PRIVACY.md](PRIVACY.md)。
- 不提交构建产物、快照、日志或本地配置。

---

## 提交信息

推荐 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>(<scope>): <subject>

<body>
```

常用 type：`feat`、`fix`、`docs`、`refactor`、`perf`、`test`、`build`、`ci`、`chore`。

示例：`fix(storage): 修正 NVMe SMART 属性解析`

---

## 许可声明

本项目以 MIT OR Apache-2.0 双许可发布。**提交即表示你同意该贡献按 MIT OR Apache-2.0 双许可授权**，且你确认自己有权提交该代码。

若你代表雇主或其他组织贡献，请确保已获得相应授权。可选做法：在提交信息中加 `Signed-off-by: Your Name <you@example.com>`（DCO）。

---

## 安全漏洞

**不要**在公开 Issue 中披露安全漏洞。请私下邮件至 **yanruoxinsu2@outlook.com**，说明影响版本、复现方式与可能的修复方向；维护者会在确认后安排修复与披露时间。

---

## 其他

- 文档改动同样欢迎，中英文档需保持一致。
- 提交前请阅读 [LEGAL_STATEMENT.md](LEGAL_STATEMENT.md) 与 [DISCLAIMER.md](DISCLAIMER.md)。
