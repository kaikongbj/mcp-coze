# Coze MCP Server Release Script
# PowerShell script for creating releases

param(
    [string]$Version = "",
    [switch]$DryRun,
    [switch]$Force,
    [switch]$Help
)

# 显示帮助信息
if ($Help) {
    Write-Host @"
Coze MCP Server Release Script

Usage: .\scripts\release.ps1 [OPTIONS]

Options:
    -Version <version>   Specify version (e.g., "0.3.0")
    -DryRun             Preview changes without executing
    -Force              Skip confirmations
    -Help               Show this help message

Examples:
    .\scripts\release.ps1 -Version "0.3.0"
    .\scripts\release.ps1 -DryRun
    .\scripts\release.ps1 -Force
"@
    exit 0
}

# 颜色输出函数
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Success {
    param([string]$Message)
    Write-ColorOutput "✅ $Message" "Green"
}

function Write-Error {
    param([string]$Message)
    Write-ColorOutput "❌ $Message" "Red"
}

function Write-Warning {
    param([string]$Message)
    Write-ColorOutput "⚠️  $Message" "Yellow"
}

function Write-Info {
    param([string]$Message)
    Write-ColorOutput "ℹ️  $Message" "Cyan"
}

function Write-Step {
    param([string]$Message)
    Write-ColorOutput "🔧 $Message" "Blue"
}

# 获取当前版本
function Get-CurrentVersion {
    $cargoToml = Get-Content "Cargo.toml"
    $versionLine = $cargoToml | Where-Object { $_ -match '^version = ' } | Select-Object -First 1
    if ($versionLine -match 'version = "([^"]*)"') {
        return $matches[1]
    }
    return "unknown"
}

# 检查先决条件
function Test-Prerequisites {
    Write-Step "Checking prerequisites..."
    
    # 检查 Git
    try {
        $gitVersion = git --version
        Write-Success "Git found: $gitVersion"
    }
    catch {
        Write-Error "Git not found. Please install Git."
        exit 1
    }
    
    # 检查是否在 Git 仓库中
    try {
        git rev-parse --git-dir | Out-Null
        Write-Success "Git repository detected"
    }
    catch {
        Write-Error "Not in a Git repository"
        exit 1
    }
    
    # 检查工作目录是否干净
    $gitStatus = git status --porcelain
    if ($gitStatus) {
        Write-Warning "Working directory has uncommitted changes:"
        Write-Host $gitStatus
        if (-not $Force) {
            $response = Read-Host "Continue anyway? (y/N)"
            if ($response -ne "y" -and $response -ne "Y") {
                Write-Info "Release cancelled"
                exit 0
            }
        }
    } else {
        Write-Success "Working directory is clean"
    }
    
    # 检查 Cargo.toml
    if (-not (Test-Path "Cargo.toml")) {
        Write-Error "Cargo.toml not found"
        exit 1
    }
    Write-Success "Cargo.toml found"
    
    Write-Success "Prerequisites check completed"
}

# 构建项目
function Invoke-Build {
    Write-Step "Building release version..."
    
    try {
        # 清理之前的构建
        cargo clean | Out-Null
        
        # 构建发布版本
        $buildOutput = cargo build --release 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Build completed successfully"
        } else {
            Write-Error "Build failed:"
            Write-Host $buildOutput
            exit 1
        }
    }
    catch {
        Write-Error "Failed to build: $_"
        exit 1
    }
}

# 运行测试
function Invoke-Tests {
    Write-Step "Running tests..."
    
    try {
        $testOutput = cargo test 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "All tests passed"
        } else {
            Write-Warning "Some tests failed:"
            Write-Host $testOutput
            if (-not $Force) {
                $response = Read-Host "Continue with release? (y/N)"
                if ($response -ne "y" -and $response -ne "Y") {
                    Write-Info "Release cancelled"
                    exit 0
                }
            }
        }
    }
    catch {
        Write-Error "Failed to run tests: $_"
        exit 1
    }
}

# 创建发布包
function New-ReleasePackage {
    param([string]$Version)
    
    Write-Step "Creating release package..."
    
    $packageName = "coze-mcp-server-v$Version-windows"
    $packageDir = "dist\$packageName"
    
    # 创建分发目录
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    New-Item -ItemType Directory -Path $packageDir -Force | Out-Null
    
    # 复制二进制文件
    $binaryPath = "target\release\coze-mcp-server.exe"
    if (-not (Test-Path $binaryPath)) {
        Write-Error "Binary not found at $binaryPath"
        exit 1
    }
    
    Copy-Item $binaryPath "$packageDir\" -Force
    
    # 复制文档文件
    $docFiles = @(
        "README.md",
        "USAGE.md", 
        "API_REFERENCE.md",
        "CHANGELOG.md",
        "RELEASE_NOTES.md"
    )
    
    foreach ($file in $docFiles) {
        if (Test-Path $file) {
            Copy-Item $file "$packageDir\" -Force
        }
    }
    
    # 复制许可证文件
    if (Test-Path "LICENSE") {
        Copy-Item "LICENSE" "$packageDir\" -Force
    }
    
    # 创建启动脚本
    $startScript = @"
@echo off
echo Starting Coze MCP Server v$Version...
echo.
echo Please make sure you have set the following environment variables:
echo   COZE_API_TOKEN=your_api_token_here
echo   COZE_DEFAULT_SPACE_ID=your_space_id_here
echo.
echo Or use command line arguments:
echo   coze-mcp-server.exe --api-key YOUR_TOKEN --space-id YOUR_SPACE_ID
echo.
pause
coze-mcp-server.exe %*
"@
    
    $startScript | Out-File -FilePath "$packageDir\start.bat" -Encoding ASCII
    
    # 创建配置示例
    $configExample = @"
# Coze MCP Server Configuration Example
# Copy this to your environment or use command line arguments

# Required: Your Coze API Token (get from https://www.coze.cn)
COZE_API_TOKEN=pat_your_token_here

# Optional: Default Space ID
COZE_DEFAULT_SPACE_ID=your_space_id_here

# Optional: API Base URL (default: https://api.coze.cn)
COZE_API_BASE_URL=https://api.coze.cn

# Optional: Log Level (debug, info, warn, error)
RUST_LOG=info
"@
    
    $configExample | Out-File -FilePath "$packageDir\config.env.example" -Encoding UTF8
    
    # 创建 Claude Desktop 配置示例
    $claudeConfig = @"
{
  "mcpServers": {
    "coze": {
      "command": "C:\\path\\to\\coze-mcp-server.exe",
      "args": [
        "--api-key",
        "pat_your_actual_token_here",
        "--space-id", 
        "your_actual_space_id"
      ]
    }
  }
}
"@
    
    $claudeConfig | Out-File -FilePath "$packageDir\claude-desktop-config.json" -Encoding UTF8
    
    # 创建压缩包
    $zipPath = "dist\$packageName.zip"
    try {
        Compress-Archive -Path "$packageDir\*" -DestinationPath $zipPath -Force
        Write-Success "Package created: $zipPath"
    }
    catch {
        Write-Warning "Failed to create zip archive: $_"
        Write-Info "Package directory available at: $packageDir"
    }
    
    # 显示包信息
    $binarySize = (Get-Item $binaryPath).Length
    $binarySizeMB = [math]::Round($binarySize / 1MB, 2)
    
    Write-Info "Package Information:"
    Write-Host "  Version: $Version" -ForegroundColor White
    Write-Host "  Binary Size: $binarySizeMB MB" -ForegroundColor White
    Write-Host "  Package: $packageName" -ForegroundColor White
    if (Test-Path $zipPath) {
        $zipSize = (Get-Item $zipPath).Length
        $zipSizeMB = [math]::Round($zipSize / 1MB, 2)
        Write-Host "  Archive Size: $zipSizeMB MB" -ForegroundColor White
    }
    
    return $zipPath
}

# 创建 Git 标签
function New-GitTag {
    param([string]$Version)
    
    Write-Step "Creating Git tag..."
    
    $tagName = "v$Version"
    
    # 检查标签是否已存在
    $existingTag = git tag -l $tagName
    if ($existingTag) {
        if ($Force) {
            Write-Warning "Tag $tagName already exists, deleting..."
            git tag -d $tagName | Out-Null
            git push origin ":refs/tags/$tagName" 2>$null | Out-Null
        } else {
            Write-Error "Tag $tagName already exists"
            Write-Info "Use -Force to overwrite existing tag"
            exit 1
        }
    }
    
    if (-not $DryRun) {
        # 创建标签
        git tag -a $tagName -m "Release version $Version" | Out-Null
        Write-Success "Created tag: $tagName"
        
        # 推送标签
        if (-not $Force) {
            $response = Read-Host "Push tag to remote? (y/N)"
            if ($response -eq "y" -or $response -eq "Y") {
                git push origin $tagName | Out-Null
                Write-Success "Pushed tag to remote"
            }
        } else {
            git push origin $tagName | Out-Null
            Write-Success "Pushed tag to remote"
        }
    } else {
        Write-Info "[DRY RUN] Would create and push tag: $tagName"
    }
}

# 更新 CHANGELOG
function Update-Changelog {
    param([string]$Version)
    
    Write-Step "Updating CHANGELOG..."
    
    if (-not (Test-Path "CHANGELOG.md")) {
        Write-Warning "CHANGELOG.md not found, skipping update"
        return
    }
    
    $currentDate = Get-Date -Format "yyyy-MM-dd"
    
    if (-not $DryRun) {
        $changelog = Get-Content "CHANGELOG.md"
        $newChangelog = @()
        $unreleased = $false
        
        foreach ($line in $changelog) {
            if ($line -match "^## \[Unreleased\]") {
                $newChangelog += $line
                $newChangelog += ""
                $newChangelog += "## [$Version] - $currentDate"
                $unreleased = $true
            } elseif ($unreleased -and $line -match "^## \[") {
                $newChangelog += ""
                $newChangelog += $line
                $unreleased = $false
            } else {
                $newChangelog += $line
            }
        }
        
        $newChangelog | Out-File -FilePath "CHANGELOG.md" -Encoding UTF8
        Write-Success "Updated CHANGELOG.md"
    } else {
        Write-Info "[DRY RUN] Would update CHANGELOG.md with version $Version"
    }
}

# 主发布流程
function Start-Release {
    $startTime = Get-Date
    
    Write-ColorOutput "🚀 Coze MCP Server Release Script" "Magenta"
    Write-Host "=" * 50
    
    # 获取版本信息
    $currentVersion = Get-CurrentVersion
    if (-not $Version) {
        $Version = $currentVersion
    }
    
    Write-Info "Release Configuration:"
    Write-Host "  Current Version: $currentVersion" -ForegroundColor White
    Write-Host "  Release Version: $Version" -ForegroundColor White
    Write-Host "  Dry Run: $DryRun" -ForegroundColor White
    Write-Host "  Force: $Force" -ForegroundColor White
    Write-Host ""
    
    try {
        # 检查先决条件
        Test-Prerequisites
        
        # 构建项目
        if (-not $DryRun) {
            Invoke-Build
            Invoke-Tests
        } else {
            Write-Info "[DRY RUN] Would build and test project"
        }
        
        # 创建发布包
        if (-not $DryRun) {
            $packagePath = New-ReleasePackage -Version $Version
        } else {
            Write-Info "[DRY RUN] Would create release package"
        }
        
        # 更新 CHANGELOG
        Update-Changelog -Version $Version
        
        # 创建 Git 标签
        New-GitTag -Version $Version
        
        # 发布完成
        $endTime = Get-Date
        $duration = $endTime - $startTime
        
        Write-Host ""
        if ($DryRun) {
            Write-Success "Dry run completed successfully!"
        } else {
            Write-Success "Release completed successfully!"
        }
        Write-Info "Total time: $($duration.TotalSeconds.ToString('F2')) seconds"
        
        if (-not $DryRun -and $packagePath -and (Test-Path $packagePath)) {
            Write-Info "Release package: $packagePath"
            Write-Info "You can now upload this package to GitHub Releases"
        }
        
    }
    catch {
        Write-Error "Release failed: $_"
        exit 1
    }
}

# 执行发布
Start-Release