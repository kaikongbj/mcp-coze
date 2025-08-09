use coze_mcp_server::api::{ListDatasetsApiResponse, ListKnowledgeBasesResponse};

#[test]
fn test_deserialize_official_shape() {
    let json = r#"{
        "code": 0,
        "msg": "Success",
        "data": {
            "total_count": 1,
            "dataset_list": [
                {
                    "dataset_id": "ds_1",
                    "name": "kb1",
                    "description": "desc",
                    "doc_count": 2,
                    "create_time": 1733817948,
                    "update_time": 1733818000,
                    "status": 1,
                    "format_type": 2,
                    "slice_count": 5,
                    "space_id": "space_x",
                    "dataset_type": 0,
                    "can_edit": true,
                    "icon_url": "https://example/icon.png",
                    "hit_count": 0,
                    "all_file_size": "0"
                }
            ]
        },
        "detail": {"logid": "LOG123"}
    }"#;
    let parsed: ListDatasetsApiResponse = serde_json::from_str(json).expect("parse official shape");
    assert_eq!(parsed.code, 0);
    let internal: ListKnowledgeBasesResponse = parsed.into_internal();
    assert_eq!(internal.total, 1);
    assert_eq!(internal.datasets.len(), 1);
    assert_eq!(internal.datasets[0].dataset_id, "ds_1");
    assert_eq!(internal.datasets[0].all_file_size, Some(0));
}

#[test]
fn test_fallback_shape() {
    // Simulate variant returning datasets & total only
    let json = r#"{
        "code": 0,
        "data": {
            "total": 2,
            "datasets": [
                {"dataset_id": "a", "name":"A", "description":"d", "doc_count":1, "create_time":1},
                {"dataset_id": "b", "name":"B", "description":"d2", "doc_count":0, "create_time":2}
            ]
        }
    }"#;
    let parsed: ListDatasetsApiResponse = serde_json::from_str(json).expect("parse fallback shape");
    let internal = parsed.into_internal();
    assert_eq!(internal.total, 2);
    assert_eq!(internal.datasets.len(), 2);
}
