#[cfg(test)]
mod tests {
    use coze_mcp_server::api::bot_models::*;
    use serde_json::json;

    #[test]
    fn test_bot_publish_status_serialization() {
        let status = BotPublishStatus::PublishedOnline;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"published_online\"");

        let status = BotPublishStatus::All;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"all\"");
    }

    #[test]
    fn test_bot_publish_status_deserialization() {
        let json_str = "\"published_online\"";
        let status: BotPublishStatus = serde_json::from_str(json_str).unwrap();
        matches!(status, BotPublishStatus::PublishedOnline);

        let json_str = "\"unpublished_draft\"";
        let status: BotPublishStatus = serde_json::from_str(json_str).unwrap();
        matches!(status, BotPublishStatus::UnpublishedDraft);
    }

    #[test]
    fn test_list_bots_request_creation() {
        let request = ListBotsRequest::new("test_workspace_123".to_string());

        assert_eq!(request.workspace_id, "test_workspace_123");
        assert!(matches!(
            request.publish_status,
            Some(BotPublishStatus::PublishedOnline)
        ));
        assert_eq!(request.connector_id, Some("1024".to_string()));
        assert_eq!(request.page_num, Some(1));
        assert_eq!(request.page_size, Some(20));
    }

    #[test]
    fn test_list_bots_request_builder_pattern() {
        let request = ListBotsRequest::new("workspace_456".to_string())
            .with_publish_status(BotPublishStatus::All)
            .with_connector_id("2048".to_string())
            .with_page(2, 50);

        assert_eq!(request.workspace_id, "workspace_456");
        assert!(matches!(
            request.publish_status,
            Some(BotPublishStatus::All)
        ));
        assert_eq!(request.connector_id, Some("2048".to_string()));
        assert_eq!(request.page_num, Some(2));
        assert_eq!(request.page_size, Some(50));
    }

    #[test]
    fn test_list_bots_request_query_params() {
        let request = ListBotsRequest::new("test%20workspace".to_string())
            .with_publish_status(BotPublishStatus::PublishedDraft)
            .with_page(3, 15);

        let query_params = request.to_query_params();

        assert!(query_params.contains("workspace_id=test%2520workspace"));
        assert!(query_params.contains("publish_status=published_draft"));
        assert!(query_params.contains("connector_id=1024"));
        assert!(query_params.contains("page_num=3"));
        assert!(query_params.contains("page_size=15"));
    }

    #[test]
    fn test_bot_info_serialization() {
        let bot = BotInfo {
            id: "bot_123".to_string(),
            name: "测试机器人".to_string(),
            icon_url: Some("https://example.com/icon.png".to_string()),
            updated_at: Some(1718289297),
            description: Some("这是一个测试机器人".to_string()),
            is_published: Some(true),
            owner_user_id: Some("user_456".to_string()),
        };

        let serialized = serde_json::to_value(&bot).unwrap();
        assert_eq!(serialized["id"], "bot_123");
        assert_eq!(serialized["name"], "测试机器人");
        assert_eq!(serialized["icon_url"], "https://example.com/icon.png");
        assert_eq!(serialized["updated_at"], 1718289297);
        assert_eq!(serialized["is_published"], true);
    }

    #[test]
    fn test_bot_info_deserialization() {
        let json = json!({
            "id": "7493066380997****",
            "name": "语音伴侣",
            "icon_url": "https://example.com/agent1***.png",
            "updated_at": 1718289297,
            "description": "语音伴侣",
            "is_published": false,
            "owner_user_id": "23423423****"
        });

        let bot: BotInfo = serde_json::from_value(json).unwrap();
        assert_eq!(bot.id, "7493066380997****");
        assert_eq!(bot.name, "语音伴侣");
        assert_eq!(
            bot.icon_url,
            Some("https://example.com/agent1***.png".to_string())
        );
        assert_eq!(bot.updated_at, Some(1718289297));
        assert_eq!(bot.description, Some("语音伴侣".to_string()));
        assert_eq!(bot.is_published, Some(false));
        assert_eq!(bot.owner_user_id, Some("23423423****".to_string()));
    }

    #[test]
    fn test_list_bots_response_deserialization() {
        let json = json!({
            "data": {
                "items": [
                    {
                        "id": "7493066380997****",
                        "name": "语音伴侣",
                        "icon_url": "https://example.com/agent1***.png",
                        "updated_at": 1718289297,
                        "description": "语音伴侣",
                        "is_published": false,
                        "owner_user_id": "23423423****"
                    }
                ],
                "total": 1
            },
            "code": 0,
            "msg": "Success",
            "detail": {
                "logid": "20241210152726467C48D89D6DB2****"
            }
        });

        let response: ListBotsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.code, 0);
        assert_eq!(response.msg, "Success");
        assert_eq!(response.data.total, 1);
        assert_eq!(response.data.items.len(), 1);

        let bot = &response.data.items[0];
        assert_eq!(bot.id, "7493066380997****");
        assert_eq!(bot.name, "语音伴侣");
        assert_eq!(bot.is_published, Some(false));
    }

    #[test]
    fn test_bot_info_with_minimal_fields() {
        let json = json!({
            "id": "minimal_bot",
            "name": "极简机器人"
        });

        let bot: BotInfo = serde_json::from_value(json).unwrap();
        assert_eq!(bot.id, "minimal_bot");
        assert_eq!(bot.name, "极简机器人");
        assert_eq!(bot.icon_url, None);
        assert_eq!(bot.updated_at, None);
        assert_eq!(bot.description, None);
        assert_eq!(bot.is_published, None);
        assert_eq!(bot.owner_user_id, None);
    }

    #[test]
    fn test_default_bot_publish_status() {
        let default_status = BotPublishStatus::default();
        assert!(matches!(default_status, BotPublishStatus::PublishedOnline));
    }
}
