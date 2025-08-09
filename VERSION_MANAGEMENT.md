# 版本管理和发布指南

本项目使用自动化的版本管理和发布流程，支持语义化版本控制和多平台构建。

## 快速开始

### 自动发布新版本

#### 使用发布脚本（推荐）

**Linux/macOS:**

```bash
# 升级补丁版本 (0.2.0 → 0.2.1)
./scripts/release.sh patch

# 升级次版本 (0.2.0 → 0.3.0)
./scripts/release.sh minor

# 升级主版本 (0.2.0 → 1.0.0)
./scripts/release.sh major

# 设置指定版本
./scripts/release.sh 1.5.0

# 预演模式（不实际执行）
./scripts/release.sh --dry-run patch
```

**Windows:**

```cmd
# 升级补丁版本
scripts\release.bat patch

# 升级次版本
scripts\release.bat minor

# 升级主版本
scripts\release.bat major

# 设置指定版本
scripts\release.bat 1.5.0

# 预演模式
scripts\release.bat /dry-run patch
```

#### 手动发布

1. **更新版本号**
   
   ```bash
   # 编辑 Cargo.toml
   version = "0.3.0"
   
   # 编辑 src/main.rs
   version: "0.3.0".into(),
   ```

2. **提交并创建标签**
   
   ```bash
   git add Cargo.toml src/main.rs
   git commit -m "chore: bump version to 0.3.0"
   git tag -a v0.3.0 -m "Release version 0.3.0"
   git push origin main
   git push origin v0.3.0
   ```

### GitHub Actions 工作流程

#### 自动触发发布

当推送版本标签时，GitHub Actions 会自动：

1. **多平台构建** - 为以下平台构建二进制文件：
   - Linux x86_64
   - Windows x86_64  
   - macOS x86_64
   - macOS ARM64

2. **运行测试** - 执行完整的测试套件

3. **创建GitHub Release** - 自动创建发布页面并上传构建产物

4. **生成变更日志** - 自动生成发布说明

#### 手动触发发布

也可以通过GitHub网页手动触发发布：

1. 访问 Actions 页面
2. 选择 "Release" 工作流程  
3. 点击 "Run workflow"
4. 输入版本号（如 1.2.3）
5. 点击运行

## 版本命名规范

我们遵循[语义化版本控制](https://semver.org/lang/zh-CN/)：

- **主版本号**：不兼容的API修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

### 版本号示例

- `0.1.0` → `0.1.1` (补丁更新，bug修复)
- `0.1.1` → `0.2.0` (次版本更新，新功能)
- `0.2.0` → `1.0.0` (主版本更新，重大变更)

## CI/CD 流程

### 持续集成 (CI)

每次推送到 `main` 或 `develop` 分支，或创建PR时都会触发：

- **代码质量检查**
  - `cargo clippy` - 代码linting
  - `cargo fmt --check` - 代码格式检查
  
- **多平台测试**
  - Linux, Windows, macOS 上运行测试套件
  
- **安全审计**
  - 使用 `cargo audit` 检查安全漏洞

### 持续部署 (CD)

版本标签推送时触发发布流程：

1. **构建阶段**
   - 多平台交叉编译
   - 缓存Cargo依赖以加速构建
   - 生成优化的发布版本

2. **测试阶段**
   - 运行完整测试套件
   - 确保发布版本稳定性

3. **发布阶段**
   - 创建GitHub Release
   - 上传构建产物
   - 自动生成变更日志

## 发布产物

每个版本会生成以下文件：

- `coze-mcp-server-v{version}-linux-x86_64`
- `coze-mcp-server-v{version}-windows-x86_64.exe`
- `coze-mcp-server-v{version}-macos-x86_64`
- `coze-mcp-server-v{version}-macos-aarch64`

## 故障排查

### 发布脚本问题

**权限错误（Linux/macOS）:**

```bash
chmod +x scripts/release.sh
```

**Git权限问题:**

```bash
# 确保有推送权限
git remote -v
# 配置SSH密钥或token
```

### GitHub Actions问题

**构建失败:**

- 检查 Actions 页面的详细日志
- 确保所有测试通过
- 检查依赖项是否有问题

**发布失败:**

- 检查是否有`GITHUB_TOKEN`权限
- 确保标签格式正确 (v1.2.3)
- 检查仓库设置中的Actions权限

### 版本冲突

**标签已存在:**

```bash
# 删除本地标签
git tag -d v1.2.3

# 删除远程标签  
git push origin :refs/tags/v1.2.3

# 或使用强制选项
./scripts/release.sh --force 1.2.3
```

## 最佳实践

1. **发布前检查**
   - 运行测试：`cargo test`
   - 检查代码质量：`cargo clippy`
   - 确保文档更新

2. **版本规划**
   - 明确变更类型（功能/修复/破坏性）
   - 适当选择版本类型（major/minor/patch）
   - 更新CHANGELOG.md

3. **发布后验证**
   - 检查GitHub Release页面
   - 测试下载的二进制文件
   - 验证版本号正确性

## 支持

如果遇到问题：

1. 查看GitHub Actions日志
2. 检查本文档的故障排查部分
3. 提交Issue描述具体问题
