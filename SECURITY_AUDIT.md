# Security Audit 说明

## Cargo Audit 配置

本项目使用 `cargo audit` 进行安全审计检查。配置文件位于 `.cargo/audit.toml`。

## 当前忽略的警告

### RUSTSEC-2024-0436: paste crate 不再维护

**状态**: 已忽略

**原因**:

- `paste` crate 是通过 `rmcp` 依赖间接引入的
- 这是一个简单的过程宏库，功能稳定
- 没有已知的安全漏洞，只是维护者停止维护
- 作为间接依赖，我们无法直接控制

**可能的解决方案**:

1. 等待 `rmcp` crate 迁移到替代品（如 `pastey`）
2. 联系 `rmcp` 维护者建议迁移
3. 继续使用当前版本（推荐，因为没有安全风险）

## 依赖关系

```text
paste v1.0.15 (proc-macro)
└── rmcp v0.5.0
    └── coze-mcp-server v0.2.1
```

## 审计命令

在 CI/CD 中自动运行：

```bash
# 安装 cargo-audit（如果未安装）
cargo install cargo-audit

# 运行审计
cargo audit
```

## 更新警告忽略列表

如果需要忽略新的警告，请编辑 `.cargo/audit.toml` 文件的 `ignore` 数组。

## 定期审查

建议定期审查忽略的警告列表，确保：

1. 忽略的警告仍然适用
2. 没有新的安全漏洞
3. 依赖项是否有更新可用
