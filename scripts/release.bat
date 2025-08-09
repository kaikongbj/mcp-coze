@echo off
setlocal enabledelayedexpansion

REM 版本管理脚本 - Windows版本
REM 自动更新版本号并创建发布

REM 设置变量
set "SCRIPT_NAME=%~nx0"
set "DRY_RUN=false"
set "FORCE=false"
set "BUMP_TYPE="

REM 显示使用说明
:show_usage
echo 版本管理脚本 - Windows版本
echo.
echo 用法: %SCRIPT_NAME% [选项] ^<版本类型^>
echo.
echo 版本类型:
echo   major    主版本号 (x.0.0)
echo   minor    次版本号 (x.y.0)
echo   patch    补丁版本号 (x.y.z)
echo   ^<版本号^>  指定版本号 (例如: 1.2.3)
echo.
echo 选项:
echo   /h, /help     显示此帮助信息
echo   /n, /dry-run  预演模式，不实际执行
echo   /f, /force    强制执行，跳过确认
echo.
echo 示例:
echo   %SCRIPT_NAME% patch              # 升级补丁版本
echo   %SCRIPT_NAME% minor              # 升级次版本
echo   %SCRIPT_NAME% major              # 升级主版本
echo   %SCRIPT_NAME% 1.5.0              # 设置为指定版本
echo   %SCRIPT_NAME% /dry-run patch     # 预演补丁版本升级
goto :eof

REM 解析命令行参数
:parse_args
if "%~1"=="" goto :check_args
if /i "%~1"=="/h" goto :show_usage
if /i "%~1"=="/help" goto :show_usage
if /i "%~1"=="/n" set "DRY_RUN=true" & shift & goto :parse_args
if /i "%~1"=="/dry-run" set "DRY_RUN=true" & shift & goto :parse_args
if /i "%~1"=="/f" set "FORCE=true" & shift & goto :parse_args
if /i "%~1"=="/force" set "FORCE=true" & shift & goto :parse_args
if "%~1" neq "" if "!BUMP_TYPE!"=="" set "BUMP_TYPE=%~1" & shift & goto :parse_args
echo 错误: 未知选项或多余参数 %~1
goto :show_usage

:check_args
if "!BUMP_TYPE!"=="" (
    echo 错误: 必须指定版本类型
    goto :show_usage
)

REM 检查Git仓库
git rev-parse --git-dir >nul 2>&1
if errorlevel 1 (
    echo 错误: 当前目录不是Git仓库
    exit /b 1
)

REM 检查Cargo.toml
if not exist "Cargo.toml" (
    echo 错误: 当前目录没有Cargo.toml文件
    exit /b 1
)

REM 获取当前版本
for /f "tokens=2 delims== " %%a in ('findstr "^version = " Cargo.toml') do (
    set "CURRENT_VERSION=%%a"
    set "CURRENT_VERSION=!CURRENT_VERSION:"=!"
)

echo 当前版本: !CURRENT_VERSION!

REM 计算新版本
call :calculate_new_version "!CURRENT_VERSION!" "!BUMP_TYPE!" NEW_VERSION
echo 新版本: !NEW_VERSION!

REM 确认操作
if "!FORCE!"=="false" if "!DRY_RUN!"=="false" (
    echo.
    echo 将执行以下操作:
    echo   • 更新 Cargo.toml 版本: !CURRENT_VERSION! → !NEW_VERSION!
    echo   • 更新 src/main.rs 版本: !CURRENT_VERSION! → !NEW_VERSION!
    echo   • 创建 Git 提交和标签: v!NEW_VERSION!
    echo.
    set /p "CONFIRM=确认继续? [y/N] "
    if /i "!CONFIRM!" neq "y" (
        echo 操作已取消
        exit /b 0
    )
)

REM 执行更新
echo 开始版本更新...
call :update_cargo_version "!NEW_VERSION!" "!DRY_RUN!"
call :update_main_version "!NEW_VERSION!" "!DRY_RUN!"
call :create_git_tag "!NEW_VERSION!" "!DRY_RUN!" "!FORCE!"

if "!DRY_RUN!"=="true" (
    echo 🔍 预演完成 - 没有实际更改
) else (
    echo 🎉 版本更新完成!
    echo 📦 新版本: !NEW_VERSION!
    echo 🏷️  Git标签: v!NEW_VERSION!
)

goto :eof

REM 计算新版本号
:calculate_new_version
set "current=%~1"
set "bump_type=%~2"

REM 解析当前版本号
for /f "tokens=1,2,3 delims=." %%a in ("%current%") do (
    set "major=%%a"
    set "minor=%%b"
    set "patch=%%c"
)

if /i "%bump_type%"=="major" (
    set /a "major+=1"
    set "minor=0"
    set "patch=0"
) else if /i "%bump_type%"=="minor" (
    set /a "minor+=1"
    set "patch=0"
) else if /i "%bump_type%"=="patch" (
    set /a "patch+=1"
) else (
    REM 检查是否是有效的版本号格式
    echo %bump_type% | findstr /r "^[0-9]*\.[0-9]*\.[0-9]*$" >nul
    if errorlevel 1 (
        echo 错误: 无效的版本类型或版本号格式: %bump_type%
        exit /b 1
    )
    set "%~3=%bump_type%"
    goto :eof
)

set "%~3=%major%.%minor%.%patch%"
goto :eof

REM 更新Cargo.toml版本
:update_cargo_version
set "new_version=%~1"
set "dry_run=%~2"

if "%dry_run%"=="true" (
    echo [DRY RUN] 将更新 Cargo.toml 版本为: %new_version%
) else (
    powershell -Command "(Get-Content Cargo.toml) -replace '^version = \".*\"', 'version = \"%new_version%\"' | Set-Content Cargo.toml"
    echo ✓ 已更新 Cargo.toml 版本为: %new_version%
)
goto :eof

REM 更新main.rs版本
:update_main_version
set "new_version=%~1"
set "dry_run=%~2"

if "%dry_run%"=="true" (
    echo [DRY RUN] 将更新 main.rs 版本为: %new_version%
) else (
    powershell -Command "(Get-Content src/main.rs) -replace 'version: \".*\"', 'version: \"%new_version%\"' | Set-Content src/main.rs"
    echo ✓ 已更新 main.rs 版本为: %new_version%
)
goto :eof

REM 创建Git标签
:create_git_tag
set "new_version=%~1"
set "dry_run=%~2"
set "force=%~3"

if "%dry_run%"=="true" (
    echo [DRY RUN] 将创建Git提交和标签: v%new_version%
    goto :eof
)

REM 检查是否有未提交的更改
git diff --quiet
if errorlevel 1 (
    echo 检测到未提交的更改，正在添加到Git...
    git add Cargo.toml src/main.rs
    git commit -m "chore: bump version to %new_version%"
    echo ✓ 已提交版本更新
)

REM 检查标签是否已存在
git tag -l | findstr "^v%new_version%$" >nul
if not errorlevel 1 (
    if "%force%"=="true" (
        echo 警告: 标签 v%new_version% 已存在，强制删除并重新创建
        git tag -d "v%new_version%"
        git push origin ":refs/tags/v%new_version%" 2>nul
    ) else (
        echo 错误: 标签 v%new_version% 已存在
        echo 使用 /force 选项强制覆盖，或选择不同的版本号
        exit /b 1
    )
)

REM 创建标签
git tag -a "v%new_version%" -m "Release version %new_version%"
echo ✓ 已创建标签: v%new_version%

REM 推送到远程
if "%force%"=="true" goto :push_remote
set /p "PUSH=是否推送到远程仓库? [y/N] "
if /i "!PUSH!"=="y" (
    :push_remote
    git push origin main
    git push origin "v%new_version%"
    echo ✓ 已推送到远程仓库
    echo 🚀 GitHub Actions 将自动构建和发布版本 %new_version%
)

goto :eof

REM 主程序入口
call :parse_args %*
