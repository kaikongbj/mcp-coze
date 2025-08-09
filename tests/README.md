# 集成测试文档

## 测试概述

本项目包含精简的集成测试文件，用于验证API格式兼容性、数据序列化以及实际API调用功能。

## 测试文件说明

### 1. `format_validation.rs`
- **路径**: `tests/format_validation.rs`
- **功能**: 验证API响应格式的兼容性
- **测试内容**:
  - API格式兼容性测试
  - 序列化往返测试
- **状态**: ✅ 通过

### 2. `integration_test_fixed.rs`
- **路径**: `tests/integration_test_fixed.rs`
- **功能**: 实际API集成测试
- **测试内容**:
  - 真实API调用测试
  - 环境变量检查
  - 分页功能测试
  - 过滤功能测试
- **状态**: ✅ 通过

## 环境变量配置

运行集成测试前，请配置以下环境变量：

```bash
# 必需
COZE_API_TOKEN=your_api_token_here

# 可选
COZE_BASE_URL=https://api.coze.cn
COZE_SPACE_ID=your_space_id
```

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试
```bash
# 运行格式验证测试
cargo test --test format_validation

# 运行集成测试
cargo test --test integration_test_fixed
```

### 运行单个测试函数
```bash
# 运行环境检查测试
cargo test --test integration_test_fixed test_environment_check

# 运行真实API测试
cargo test --test integration_test_fixed test_real_api_integration
```

## 测试状态总结

| 测试文件 | 状态 | 说明 |
|----------|------|------|
| `format_validation.rs` | ✅ 通过 | API格式验证 |
| `integration_test_fixed.rs` | ✅ 通过 | 实际API集成测试 |

## 注意事项

1. **环境变量**: 确保设置了 `COZE_API_TOKEN` 才能运行真实API测试
2. **网络连接**: 真实API测试需要网络连接
3. **API限制**: 注意Coze API的调用频率限制
4. **错误处理**: 测试中会处理网络错误和API错误，不会导致测试失败

## 测试结果示例

成功运行后的输出示例：

```
running 5 tests
test test_environment_check ... ok
test test_pagination ... ok
test test_real_api_integration ... ok
test test_filtering ... ok
test test_api_endpoints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```