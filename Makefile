# Makefile for coze-mcp-server

# 变量定义
BINARY_NAME := coze-mcp-server
VERSION := $(shell grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release

# 默认目标
.PHONY: all
all: build

# 构建目标
.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

# 测试目标
.PHONY: test
test:
	cargo test

.PHONY: test-verbose
test-verbose:
	cargo test -- --nocapture

# 代码质量检查
.PHONY: fmt
fmt:
	cargo fmt

.PHONY: fmt-check
fmt-check:
	cargo fmt -- --check

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: audit
audit:
	cargo audit

# 清理
.PHONY: clean
clean:
	cargo clean

# 开发便利目标
.PHONY: dev
dev: fmt clippy test

.PHONY: ci
ci: fmt-check clippy test audit

# 版本管理
.PHONY: version
version:
	@echo "Current version: $(VERSION)"

.PHONY: version-patch
version-patch:
	@if [ -x "scripts/release.sh" ]; then \
		scripts/release.sh patch; \
	else \
		echo "Release script not found or not executable"; \
	fi

.PHONY: version-minor
version-minor:
	@if [ -x "scripts/release.sh" ]; then \
		scripts/release.sh minor; \
	else \
		echo "Release script not found or not executable"; \
	fi

.PHONY: version-major
version-major:
	@if [ -x "scripts/release.sh" ]; then \
		scripts/release.sh major; \
	else \
		echo "Release script not found or not executable"; \
	fi

# 安装目标
.PHONY: install
install: release
	cargo install --path .

# 文档
.PHONY: doc
doc:
	cargo doc --open

# 运行
.PHONY: run
run:
	cargo run

.PHONY: run-release
run-release: release
	$(RELEASE_DIR)/$(BINARY_NAME)

# 帮助
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build          - Build debug version"
	@echo "  release        - Build release version"
	@echo "  test           - Run tests"
	@echo "  test-verbose   - Run tests with output"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  clippy         - Run clippy linter"
	@echo "  audit          - Security audit"
	@echo "  clean          - Clean build artifacts"
	@echo "  dev            - Run dev checks (fmt + clippy + test)"
	@echo "  ci             - Run CI checks (fmt-check + clippy + test + audit)"
	@echo "  version        - Show current version"
	@echo "  version-patch  - Bump patch version"
	@echo "  version-minor  - Bump minor version"
	@echo "  version-major  - Bump major version"
	@echo "  install        - Install binary"
	@echo "  doc            - Generate and open documentation"
	@echo "  run            - Run debug version"
	@echo "  run-release    - Run release version"
	@echo "  help           - Show this help"
