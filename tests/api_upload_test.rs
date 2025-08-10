//! Minimal test retained after refactor: ensure KnowledgeConfig defaults are stable.

use coze_mcp_server::knowledge::KnowledgeConfig;

#[test]
fn test_knowledge_config_default() {
    let cfg = KnowledgeConfig::default();
    assert_eq!(cfg.chunk_size, 800);
    assert_eq!(cfg.chunk_overlap, 100);
    // max_file_size removed in refactor
}
