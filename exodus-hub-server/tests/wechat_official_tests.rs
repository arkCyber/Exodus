//! Unit tests for wechat_official module

#[cfg(test)]
mod tests {
    use exodus_hub_server::wechat_official::{WeChatOfficialService, OfficialAccount, OfficialMessage};
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;
    use tempfile::TempDir;

    fn create_test_service() -> (WeChatOfficialService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wechat_official.db");
        let service = WeChatOfficialService::new(db_path).unwrap();
        (service, temp_dir)
    }

    #[test]
    fn test_add_account() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx1234567890", "Test Account", Some("avatar_url".to_string()), Some("Test Description".to_string())).unwrap();

        assert_eq!(account.app_id, "wx1234567890");
        assert_eq!(account.name, "Test Account");
        assert!(!account.account_id.is_empty());
    }

    #[test]
    fn test_get_account_by_app_id() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx9876543210", "Test Account 2", None, None).unwrap();

        let retrieved = service.get_account_by_app_id("wx9876543210").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().account_id, account.account_id);
    }

    #[test]
    fn test_get_all_accounts() {
        let (mut service, _temp_dir) = create_test_service();
        service.add_account("wx1111111111", "Account 1", None, None).unwrap();
        service.add_account("wx2222222222", "Account 2", None, None).unwrap();

        let accounts = service.get_all_accounts().unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[test]
    fn test_store_message() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx3333333333", "Account 3", None, None).unwrap();

        let message = OfficialMessage {
            message_id: Uuid::new_v4().to_string(),
            account_id: account.account_id.clone(),
            message_type: "text".to_string(),
            content: "Hello from WeChat".to_string(),
            media_url: None,
            timestamp: Utc::now(),
            read: false,
        };

        service.store_message(message.clone()).unwrap();

        let messages = service.get_account_messages(&account.account_id, 10).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello from WeChat");
    }

    #[test]
    fn test_subscribe_user() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx4444444444", "Account 4", None, None).unwrap();

        service.subscribe_user(&account.account_id, "user_123").unwrap();

        let subscribers = service.get_subscribers(&account.account_id).unwrap();
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], "user_123");
    }

    #[test]
    fn test_unsubscribe_user() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx5555555555", "Account 5", None, None).unwrap();

        service.subscribe_user(&account.account_id, "user_456").unwrap();
        service.unsubscribe_user(&account.account_id, "user_456").unwrap();

        let subscribers = service.get_subscribers(&account.account_id).unwrap();
        assert_eq!(subscribers.len(), 0);
    }

    #[test]
    fn test_mark_message_read() {
        let (mut service, _temp_dir) = create_test_service();
        let account = service.add_account("wx6666666666", "Account 6", None, None).unwrap();

        let message = OfficialMessage {
            message_id: Uuid::new_v4().to_string(),
            account_id: account.account_id.clone(),
            message_type: "text".to_string(),
            content: "Test message".to_string(),
            media_url: None,
            timestamp: Utc::now(),
            read: false,
        };

        service.store_message(message.clone()).unwrap();
        service.mark_message_read(&message.message_id).unwrap();

        let messages = service.get_account_messages(&account.account_id, 10).unwrap();
        assert!(messages[0].read);
    }
}
