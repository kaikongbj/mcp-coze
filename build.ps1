# PowerShell 构建脚本 for coze-mcp-server
param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(Position=1)]
    [string]$Version = ""
)

# 获取当前版本
function Get-CurrentVersion {
    $cargoToml = Get-Content "Cargo.toml"
    $versionLine = $cargoToml | Where-Object { $_ -match '^version = ' } | Select-Object -First 1
    if ($versionLine -match 'version = "([^"]*)"') {
        return $matches[1]
    }
    return "unknown"
}

# 显示帮助信息
function Show-Help {
    Write-Host "coze-mcp-server 构建脚本"
    Write-Host ""
    Write-Host "用法: .\build.ps1 [命令] [参数]"
    Write-Host ""
    Write-Host "命令:"
    Write-Host "  build          - 构建调试版本"
    Write-Host "  release        - 构建发布版本"
    Write-Host "  test           - 运行测试"
    Write-Host "  test-verbose   - 运行详细测试"
    Write-Host "  fmt            - 格式化代码"
    Write-Host "  fmt-check      - 检查代码格式"
    Write-Host "  clippy         - 运行代码检查"
    Write-Host "  audit          - 安全审计"
    Write-Host "  clean          - 清理构建产物"
    Write-Host "  dev            - 开发检查 (fmt + clippy + test)"
    Write-Host "  ci             - CI检查 (fmt-check + clippy + test + audit)"
    Write-Host "  version        - 显示当前版本"
    Write-Host "  version-patch  - 升级补丁版本"
    Write-Host "  version-minor  - 升级次版本"
    Write-Host "  version-major  - 升级主版本"
    Write-Host "  install        - 安装二进制文件"
    Write-Host "  doc            - 生成并打开文档"
    Write-Host "  run            - 运行调试版本"
    Write-Host "  run-release    - 运行发布版本"
    Write-Host "  help           - 显示此帮助"
    Write-Host ""
    Write-Host "示例:"
    Write-Host "  .\build.ps1 build"
    Write-Host "  .\build.ps1 release"
    Write-Host "  .\build.ps1 version-patch"
}

# 执行命令
switch ($Command.ToLower()) {
    "build" {
        Write-Host "🔨 构建调试版本..." -ForegroundColor Blue
        cargo build
    }
    
    "release" {
        Write-Host "🔨 构建发布版本..." -ForegroundColor Blue
        cargo build --release
    }
    
    "test" {
        Write-Host "🧪 运行测试..." -ForegroundColor Blue
        cargo test
    }
    
    "test-verbose" {
        Write-Host "🧪 运行详细测试..." -ForegroundColor Blue
        cargo test -- --nocapture
    }
    
    "fmt" {
        Write-Host "🎨 格式化代码..." -ForegroundColor Blue
        cargo fmt
    }
    
    "fmt-check" {
        Write-Host "🎨 检查代码格式..." -ForegroundColor Blue
        cargo fmt -- --check
    }
    
    "clippy" {
        Write-Host "📎 运行代码检查..." -ForegroundColor Blue
        cargo clippy -- -D warnings
    }
    
    "audit" {
        Write-Host "🔍 安全审计..." -ForegroundColor Blue
        cargo audit
    }
    
    "clean" {
        Write-Host "🧹 清理构建产物..." -ForegroundColor Blue
        cargo clean
    }
    
    "dev" {
        Write-Host "🛠️ 开发检查..." -ForegroundColor Blue
        Write-Host "  1. 格式化代码..." -ForegroundColor Gray
        cargo fmt
        Write-Host "  2. 代码检查..." -ForegroundColor Gray
        cargo clippy -- -D warnings
        Write-Host "  3. 运行测试..." -ForegroundColor Gray
        cargo test
        Write-Host "✅ 开发检查完成" -ForegroundColor Green
    }
    
    "ci" {
        Write-Host "🚀 CI检查..." -ForegroundColor Blue
        Write-Host "  1. 检查代码格式..." -ForegroundColor Gray
        cargo fmt -- --check
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        
        Write-Host "  2. 代码检查..." -ForegroundColor Gray
        cargo clippy -- -D warnings
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        
        Write-Host "  3. 运行测试..." -ForegroundColor Gray
        cargo test
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        
        Write-Host "  4. 安全审计..." -ForegroundColor Gray
        cargo audit
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        
        Write-Host "✅ CI检查完成" -ForegroundColor Green
    }
    
    "version" {
        $currentVersion = Get-CurrentVersion
        Write-Host "📦 当前版本: $currentVersion" -ForegroundColor Blue
    }
    
    "version-patch" {
        Write-Host "📈 升级补丁版本..." -ForegroundColor Blue
        if (Test-Path "scripts\release.bat") {
            & "scripts\release.bat" patch
        } else {
            Write-Host "❌ 发布脚本未找到" -ForegroundColor Red
        }
    }
    
    "version-minor" {
        Write-Host "📈 升级次版本..." -ForegroundColor Blue
        if (Test-Path "scripts\release.bat") {
            & "scripts\release.bat" minor
        } else {
            Write-Host "❌ 发布脚本未找到" -ForegroundColor Red
        }
    }
    
    "version-major" {
        Write-Host "📈 升级主版本..." -ForegroundColor Blue
        if (Test-Path "scripts\release.bat") {
            & "scripts\release.bat" major
        } else {
            Write-Host "❌ 发布脚本未找到" -ForegroundColor Red
        }
    }
    
    "install" {
        Write-Host "📦 安装二进制文件..." -ForegroundColor Blue
        cargo build --release
        cargo install --path .
    }
    
    "doc" {
        Write-Host "📚 生成文档..." -ForegroundColor Blue
        cargo doc --open
    }
    
    "run" {
        Write-Host "🚀 运行调试版本..." -ForegroundColor Blue
        cargo run
    }
    
    "run-release" {
        Write-Host "🚀 运行发布版本..." -ForegroundColor Blue
        cargo build --release
        & "target\release\coze-mcp-server.exe"
    }
    
    "help" {
        Show-Help
    }
    
    default {
        Write-Host "❌ 未知命令: $Command" -ForegroundColor Red
        Write-Host ""
        Show-Help
        exit 1
    }
}
