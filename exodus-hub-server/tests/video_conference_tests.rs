//! Unit tests for video_conference module

#[cfg(test)]
mod tests {
    use exodus_hub_server::video_conference::{VideoConferenceService, ConferenceRoom, Participant};
    use tempfile::TempDir;

    fn create_test_service() -> (VideoConferenceService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_video_conference.db");
        let service = VideoConferenceService::new(db_path).unwrap();
        (service, temp_dir)
    }

    #[test]
    fn test_create_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Test Meeting", "host_1", 10).unwrap();

        assert_eq!(room.name, "Test Meeting");
        assert_eq!(room.host_id, "host_1");
        assert_eq!(room.max_participants, 10);
        assert_eq!(room.status, "pending");
        assert!(!room.room_id.is_empty());
    }

    #[test]
    fn test_get_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 2", "host_2", 20).unwrap();

        let retrieved = service.get_room(&room.room_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().room_id, room.room_id);
    }

    #[test]
    fn test_join_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 3", "host_3", 5).unwrap();

        let participant = service.join_room(&room.room_id, "user_1", "Alice", "participant").unwrap();

        assert_eq!(participant.user_id, "user_1");
        assert_eq!(participant.display_name, "Alice");
        assert_eq!(participant.role, "participant");
        assert!(participant.audio_enabled);
        assert!(participant.video_enabled);
    }

    #[test]
    fn test_get_participants() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 4", "host_4", 10).unwrap();

        service.join_room(&room.room_id, "user_1", "Alice", "participant").unwrap();
        service.join_room(&room.room_id, "user_2", "Bob", "participant").unwrap();

        let participants = service.get_participants(&room.room_id).unwrap();
        assert_eq!(participants.len(), 2);
    }

    #[test]
    fn test_leave_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 5", "host_5", 10).unwrap();
        let participant = service.join_room(&room.room_id, "user_1", "Charlie", "participant").unwrap();

        service.leave_room(&room.room_id, "user_1").unwrap();

        let participants = service.get_participants(&room.room_id).unwrap();
        assert_eq!(participants.len(), 0);
    }

    #[test]
    fn test_update_media_state() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 6", "host_6", 10).unwrap();
        let participant = service.join_room(&room.room_id, "user_1", "David", "participant").unwrap();

        service.update_media_state(&participant.participant_id, false, true).unwrap();

        let participants = service.get_participants(&room.room_id).unwrap();
        assert!(!participants[0].audio_enabled);
        assert!(participants[0].video_enabled);
    }

    #[test]
    fn test_start_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 7", "host_7", 10).unwrap();

        service.start_room(&room.room_id).unwrap();

        let updated = service.get_room(&room.room_id).unwrap().unwrap();
        assert_eq!(updated.status, "active");
        assert!(updated.started_at.is_some());
    }

    #[test]
    fn test_end_room() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 8", "host_8", 10).unwrap();

        service.start_room(&room.room_id).unwrap();
        service.end_room(&room.room_id).unwrap();

        let updated = service.get_room(&room.room_id).unwrap().unwrap();
        assert_eq!(updated.status, "ended");
        assert!(updated.ended_at.is_some());
    }

    #[test]
    fn test_create_invite() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 9", "host_9", 10).unwrap();

        let invite_code = service.create_invite(&room.room_id, "host_9", Some("user_invite"), 24).unwrap();

        assert!(!invite_code.is_empty());
        assert!(invite_code.len() >= 30 && invite_code.len() <= 32);
    }

    #[test]
    fn test_validate_invite() {
        let (mut service, _temp_dir) = create_test_service();
        let room = service.create_room("Meeting 10", "host_10", 10).unwrap();

        let invite_code = service.create_invite(&room.room_id, "host_10", Some("user_invite"), 24).unwrap();

        let room_id = service.validate_invite(&invite_code).unwrap();
        assert!(room_id.is_some());
        assert_eq!(room_id.unwrap(), room.room_id);
    }
}
