use coze_mcp_server::api::client::CozeApiClient;
use coze_mcp_server::api::error::ApiError;
use serde_json::json;
use std::sync::Arc;
// CozeTools not publicly exported; test focuses on request shape via model instead.
use coze_mcp_server::api::knowledge_models::{
    ChunkStrategyCn, DocumentBaseCn, KnowledgeDocumentUploadRequestCn, SourceInfo,
};

#[test]
fn dry_run_request_shape_basic() -> Result<(), ApiError> {
    let doc = DocumentBaseCn {
        name: "abc.txt".into(),
        source_info: SourceInfo::file_base64("QUJD".into(), "txt".into()),
        caption: None,
        update_rule: None,
    };
    let chunk = ChunkStrategyCn::text("\n\n".into(), 800, 1);
    let req = KnowledgeDocumentUploadRequestCn {
        dataset_id: "dataset123".into(),
        document_bases: vec![doc],
        chunk_strategy: chunk,
        format_type: 0,
    };
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v.get("dataset_id").unwrap().as_str().unwrap(), "dataset123");
    let db = v.get("document_bases").unwrap().as_array().unwrap();
    assert_eq!(db.len(), 1);
    let si = db[0].get("source_info").unwrap();
    assert!(si.get("file_base64").is_some());
    assert_eq!(si.get("file_type").unwrap().as_str().unwrap(), "txt");
    let cs = v.get("chunk_strategy").unwrap();
    assert_eq!(cs.get("chunk_type").unwrap().as_i64().unwrap(), 1);
    assert_eq!(v.get("format_type").unwrap().as_i64().unwrap(), 0);
    Ok(())
}
