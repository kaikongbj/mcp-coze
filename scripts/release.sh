#!/bin/bash

# 版本管理脚本 - 自动更新版本号并创建发布

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# 显示使用说明
show_usage() {
    echo "版本管理脚本"
    echo ""
    echo "用法: $0 [选项] <版本类型>"
    echo ""
    echo "版本类型:"
    echo "  major    主版本号 (x.0.0)"
    echo "  minor    次版本号 (x.y.0)"
    echo "  patch    补丁版本号 (x.y.z)"
    echo "  <版本号>  指定版本号 (例如: 1.2.3)"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示此帮助信息"
    echo "  -n, --dry-run  预演模式，不实际执行"
    echo "  -f, --force    强制执行，跳过确认"
    echo ""
    echo "示例:"
    echo "  $0 patch              # 升级补丁版本"
    echo "  $0 minor              # 升级次版本" 
    echo "  $0 major              # 升级主版本"
    echo "  $0 1.5.0              # 设置为指定版本"
    echo "  $0 --dry-run patch    # 预演补丁版本升级"
}

# 获取当前版本
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# 解析版本号
parse_version() {
    local version=$1
    echo $version | sed 's/\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\).*/\1 \2 \3/'
}

# 计算新版本
calculate_new_version() {
    local current_version=$1
    local bump_type=$2
    
    read major minor patch <<< $(parse_version $current_version)
    
    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            # 直接指定版本号
            if [[ $bump_type =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                echo $bump_type
                return
            else
                print_message $RED "错误: 无效的版本类型或版本号格式: $bump_type"
                exit 1
            fi
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# 更新Cargo.toml中的版本
update_cargo_version() {
    local new_version=$1
    local dry_run=$2
    
    if [ "$dry_run" = true ]; then
        print_message $YELLOW "[DRY RUN] 将更新 Cargo.toml 版本为: $new_version"
    else
        sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        rm -f Cargo.toml.bak
        print_message $GREEN "✓ 已更新 Cargo.toml 版本为: $new_version"
    fi
}

# 更新main.rs中的版本
update_main_version() {
    local new_version=$1
    local dry_run=$2
    
    if [ "$dry_run" = true ]; then
        print_message $YELLOW "[DRY RUN] 将更新 main.rs 版本为: $new_version"
    else
        sed -i.bak "s/version: \".*\"/version: \"$new_version\"/" src/main.rs
        rm -f src/main.rs.bak
        print_message $GREEN "✓ 已更新 main.rs 版本为: $new_version"
    fi
}

# 创建Git标签和提交
create_git_tag() {
    local new_version=$1
    local dry_run=$2
    local force=$3
    
    if [ "$dry_run" = true ]; then
        print_message $YELLOW "[DRY RUN] 将创建Git提交和标签: v$new_version"
        return
    fi
    
    # 检查是否有未提交的更改
    if ! git diff --quiet; then
        print_message $YELLOW "检测到未提交的更改，正在添加到Git..."
        git add Cargo.toml src/main.rs
        git commit -m "chore: bump version to $new_version"
        print_message $GREEN "✓ 已提交版本更新"
    fi
    
    # 检查标签是否已存在
    if git tag -l | grep -q "^v$new_version$"; then
        if [ "$force" = true ]; then
            print_message $YELLOW "警告: 标签 v$new_version 已存在，强制删除并重新创建"
            git tag -d "v$new_version"
            git push origin ":refs/tags/v$new_version" 2>/dev/null || true
        else
            print_message $RED "错误: 标签 v$new_version 已存在"
            print_message $YELLOW "使用 --force 选项强制覆盖，或选择不同的版本号"
            exit 1
        fi
    fi
    
    # 创建标签
    git tag -a "v$new_version" -m "Release version $new_version"
    print_message $GREEN "✓ 已创建标签: v$new_version"
    
    # 推送到远程
    if [ "$force" = true ] || [ "$DRY_RUN" != true ]; then
        read -p "是否推送到远程仓库? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git push origin main
            git push origin "v$new_version"
            print_message $GREEN "✓ 已推送到远程仓库"
            print_message $BLUE "🚀 GitHub Actions 将自动构建和发布版本 $new_version"
        fi
    fi
}

# 主函数
main() {
    local dry_run=false
    local force=false
    local bump_type=""
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -n|--dry-run)
                dry_run=true
                shift
                ;;
            -f|--force)
                force=true
                shift
                ;;
            -*)
                print_message $RED "错误: 未知选项 $1"
                show_usage
                exit 1
                ;;
            *)
                if [ -z "$bump_type" ]; then
                    bump_type=$1
                else
                    print_message $RED "错误: 多余的参数 $1"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # 检查是否提供了版本类型
    if [ -z "$bump_type" ]; then
        print_message $RED "错误: 必须指定版本类型"
        show_usage
        exit 1
    fi
    
    # 检查是否在Git仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_message $RED "错误: 当前目录不是Git仓库"
        exit 1
    fi
    
    # 检查是否有Cargo.toml文件
    if [ ! -f "Cargo.toml" ]; then
        print_message $RED "错误: 当前目录没有Cargo.toml文件"
        exit 1
    fi
    
    # 获取当前版本
    current_version=$(get_current_version)
    print_message $BLUE "当前版本: $current_version"
    
    # 计算新版本
    new_version=$(calculate_new_version $current_version $bump_type)
    print_message $BLUE "新版本: $new_version"
    
    # 确认操作
    if [ "$force" != true ] && [ "$dry_run" != true ]; then
        echo
        print_message $YELLOW "将执行以下操作:"
        echo "  • 更新 Cargo.toml 版本: $current_version → $new_version"
        echo "  • 更新 src/main.rs 版本: $current_version → $new_version"
        echo "  • 创建 Git 提交和标签: v$new_version"
        echo
        read -p "确认继续? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_message $YELLOW "操作已取消"
            exit 0
        fi
    fi
    
    # 执行更新
    print_message $BLUE "开始版本更新..."
    update_cargo_version $new_version $dry_run
    update_main_version $new_version $dry_run
    create_git_tag $new_version $dry_run $force
    
    if [ "$dry_run" = true ]; then
        print_message $YELLOW "🔍 预演完成 - 没有实际更改"
    else
        print_message $GREEN "🎉 版本更新完成!"
        print_message $BLUE "📦 新版本: $new_version"
        print_message $BLUE "🏷️  Git标签: v$new_version"
    fi
}

# 运行主函数
main "$@"
