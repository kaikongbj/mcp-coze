use coze_mcp_server::api::knowledge_models::{KnowledgeDocumentUploadRequestCn, DocumentBaseCn, SourceInfo, ChunkStrategyCn};

#[test]
fn test_upload_request_shape() {
    let doc = DocumentBaseCn { name: "test.txt".into(), source_info: SourceInfo::file_base64("Zm9v".into(), "txt".into()), caption: None, update_rule: None };
    let chunk = ChunkStrategyCn::text("\n\n".into(), 800, 1);
    let req = KnowledgeDocumentUploadRequestCn { dataset_id: "123".into(), document_bases: vec![doc], chunk_strategy: chunk, format_type: 0 };
    let v = serde_json::to_value(&req).expect("serialize");
    // Basic field presence assertions
    assert!(v.get("dataset_id").is_some());
    assert!(v.get("document_bases").and_then(|d| d.as_array()).unwrap()[0].get("name").is_some());
    assert!(v.get("document_bases").and_then(|d| d.as_array()).unwrap()[0].get("source_info").is_some());
    let cs = v.get("chunk_strategy").unwrap();
    assert!(cs.get("chunk_type").is_some());
    assert_eq!(v.get("format_type").unwrap().as_i64().unwrap(), 0);
}
