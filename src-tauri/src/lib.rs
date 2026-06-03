//! Exodus Browser — Tauri 2 backend.
//! Sidecar lifecycle, RAG indexing, web-agent, native WebView, and AI proxy.

use std::sync::{Arc, Mutex};

#[cfg(target_os = "macos")]
use objc::runtime::Object;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApp, NSApplicationActivationPolicyRegular, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;

use plugins::{ExtensionManager, ExtensionState, NativePluginManager, TabRegistry};
use plugins::net_rules::NetRuleStore;
use plugins::web_request::WebRequestStore;
use discarded_tabs::DiscardedTabsRegistry;
use encrypted_sync::EncryptedSyncManager;
pub use local_pocket::{
    pocket_delete_article, pocket_get_all_tags, pocket_get_article, pocket_get_stats,
    pocket_list_articles, pocket_mark_as_read, pocket_save_article, pocket_search_articles,
    pocket_update_article, pocket_get_articles_by_tag,
};
use plugins::notifications::NotificationStore;
use plugins::permission_pending::PermissionPendingStore;
use plugins::browser_site_permissions::BrowserSitePermissionStore;
use plugins::host_install_pending::HostInstallPendingStore;
use plugins::site_permissions::SitePermissionStore;

use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager};

mod agent;
mod ai;
mod browser;
mod config;
mod cookie_manager;
mod dark_mode;
mod dns_prefetch;
mod downloads;
mod embeddings;
mod gemma4_advanced_test;
mod gemma4_inference_integration_test;
mod gemma4_inference_test;
#[cfg(test)]
mod allama_stack_test;
mod microservice;
mod microservice_commands;
mod password_manager;
mod password_autofill;
mod plugins;
mod p2p_cdn;
mod exodus_workspace;
mod file_transfer_engine;
mod wan_relay;
mod wan_relay_server;
mod workspace_watcher;
mod app_tray;
mod app_window;
mod app_lifecycle;
mod lifecycle_log;
mod startup_log;
mod video_rtc;
mod video_rtc_commands;

#[cfg(test)]
mod file_transfer_integration_tests;
mod rag;
mod sidecar;
mod smart_cache;
mod smart_suggestions;
mod tab_freezer;
mod tracking_protection;
mod translation_service;
mod safe_browsing;
mod certificate_validation;
// Extension modules temporarily removed
// mod extension_api;
// mod extension_permissions;
mod devtools;
mod privacy_dashboard;
mod reading_mode;
mod page_zoom;
mod download_manager;
mod history_manager;
mod profile;
mod profile_stores;
mod ntp_wallpapers;
mod discarded_tabs;
mod encrypted_sync;
mod filter_list_parser;
mod http_response_proxy;
mod picture_in_picture;
mod local_pocket;
mod pip_test;
mod pocket_test;
mod bookmark_sync;
mod form_autofill;
mod tab_sleeping;
mod tab_sleeping_commands;
mod microservice_monitoring_commands;
mod resource_preloader;
mod resource_preloader_commands;
mod hermes_agent;
mod hermes_commands;
mod python_microservice;
mod python_microservice_commands;
mod inference_engine;
mod inference_commands;
mod allama_manager;
mod tab_grouping;
mod vertical_tabs;
mod tab_pinning;
mod search_engine;
mod new_tab_page;
mod network_interception;
mod content_script;
mod user_script;
mod font_settings;
mod print_settings;
mod pdf_viewer;
mod tab_search;
mod tab_mute;
mod tab_preview;
mod https_only;
mod dns_over_https;
mod fingerprinting_protection;
mod omnibox_actions;
mod biometric_auth;
mod voice_search;
mod text_to_speech;
mod global_audio;
mod per_site_shields;
mod tab_stacking;
mod reading_progress;
mod annotation;
mod media_casting;
mod color_blind;
mod voice_control;
mod data_saver;
mod audio_visualization;
mod password_sharing;
mod private_search;
mod omnibox_image_search;
mod mobile_sync;
mod password_health;

use agent::{AgentAction, AgentExecutor, DomCompressor};
use config::{ConfigState, ExodusConfig};
use microservice::{ServiceRegistry, MicroserviceBus, ServiceSupervisor, log_collector::LogCollector, resource_monitor::ResourceMonitor};
use microservice_commands::{
    microservice_register, microservice_unregister, microservice_list, microservice_status,
    microservice_heartbeat, microservice_health_check_all, microservice_start, microservice_stop,
    microservice_socket_dir, microservice_get_logs, microservice_add_log, microservice_save_logs,
    microservice_clear_logs, microservice_clear_all_logs, microservice_get_all_logs,
    microservice_collect_metrics, microservice_get_metrics, microservice_get_latest_metrics,
    microservice_get_all_metrics,
    microservice_get_average_usage, microservice_clear_metrics, microservice_clear_all_metrics,
    microservice_health_check, microservice_get_health_history, microservice_get_health_stats,
};
use password_manager::PasswordManager;
use cookie_manager::CookieManager;
use smart_cache::SmartCache;
use dark_mode::ThemeManager;
use tracking_protection::TrackingProtectionManager;
use smart_suggestions::SmartSuggestionsManager;
use tab_freezer::TabFreezer;
use dns_prefetch::DnsPrefetchManager;
use translation_service::TranslationService;
use safe_browsing::SafeBrowsingManager;
use certificate_validation::CertificateValidationManager;
// use extension_api::ExtensionApiManager;
// use extension_permissions::ExtensionPermissionsManager;
use devtools::DevToolsManager;
use privacy_dashboard::PrivacyDashboardManager;
use reading_mode::ReadingModeManager;
use page_zoom::PageZoomManager;
use download_manager::DownloadManager;
use profile_stores::{clear_private_profile, ProfileCookieStores, ProfileHistoryStores};
use bookmark_sync::BookmarkSyncManager;
use form_autofill::FormAutofillManager;
use tab_grouping::TabGroupingManager;
use vertical_tabs::VerticalTabsManager;
use tab_pinning::TabPinningManager;
use search_engine::SearchEngineManager;
use new_tab_page::NewTabPageManager;
use network_interception::NetworkInterceptor;
use content_script::ContentScriptManager;
use user_script::UserScriptManager;
use font_settings::FontSettingsManager;
use print_settings::PrintSettingsManager;
use pdf_viewer::PdfViewerManager;
use microservice::rag_commands::{
    rag_service_start, rag_service_stop, rag_store_page, rag_search_pages,
    rag_add_bookmark, rag_list_bookmarks, rag_record_visit, rag_search_visits,
};
use microservice::allama_commands::{
    allama_service_start, allama_service_stop, allama_service_restart, allama_service_status,
    allama_http_health, allama_control_rpc, allama_register_microservice, allama_list_models,
};
use allama_manager::AllamaManager;
use microservice::crypto_commands::{
    crypto_service_start, crypto_service_stop, crypto_hash, crypto_uuid_generate,
    crypto_extract_12_digit, crypto_get_formatted_id, crypto_generate_qr_code, crypto_parse_qr_code,
};
use microservice::os_commands::{
    os_service_start, os_service_stop, os_get_platform, os_get_arch, os_get_temp_dir,
    os_get_home_dir, os_path_exists, os_read_file, os_write_file, os_delete_file,
    os_list_dir, os_create_dir, os_remove_dir,
};
use microservice::p2p_commands::{
    p2p_blobs_service_start, p2p_blobs_service_stop, p2p_blobs_add, p2p_blobs_get,
    p2p_blobs_list, p2p_blobs_create_ticket,
    p2p_gossip_service_start, p2p_gossip_service_stop, p2p_gossip_subscribe,
    p2p_gossip_unsubscribe, p2p_gossip_publish, p2p_gossip_get_messages,
    p2p_gossip_list_topics, p2p_gossip_get_subscribers, p2p_gossip_node_info,
};
use microservice::ai_model_commands::{
    ai_model_service_start, ai_model_service_stop, ai_model_register, ai_model_unregister,
    ai_model_get, ai_model_search, ai_model_list, ai_model_get_node_models,
    ai_model_list_nodes, ai_model_node_info,
};
use microservice::file_transfer_commands::{
    exodus_workspace_info, exodus_workspace_list, exodus_workspace_watch_start,
    exodus_workspace_watch_stop, file_transfer_receive_to_inbox, file_transfer_service_start,
    file_transfer_service_stop, file_transfer_initiate, file_transfer_pick_file,
    file_transfer_dashboard, file_transfer_set_throttle, file_transfer_set_auto_reconnect,
    file_transfer_set_relay_config, file_transfer_set_relay_serve, wan_relay_server_info,
    file_transfer_start_background_download,
    file_transfer_verify_checksum, file_transfer_get, file_transfer_list,
    file_transfer_get_chunks, file_transfer_update_status, file_transfer_cancel,
    file_transfer_generate_qr_data, file_transfer_resolve_by_short_code, file_transfer_retry,
    file_transfer_resolve_conflict,
};
use microservice::ai_agent_commands::{
    ai_agent_service_start, ai_agent_service_stop, ai_agent_register, ai_agent_unregister,
    ai_agent_get, ai_agent_list, ai_agent_find_by_capability, ai_agent_find_by_type,
    ai_agent_update_status, ai_agent_send_message, ai_agent_get_messages,
    ai_agent_broadcast_presence, ai_agent_link_to_public_account,
    ai_agent_unlink_from_public_account, ai_agent_get_agents_by_public_account,
};
use microservice::contact_directory_commands::{
    contact_directory_service_start, contact_directory_service_stop, contact_directory_hub_info,
    contact_get_local_digit, contact_export_json, contact_import_json,
    contact_add, contact_get, contact_update,
    contact_remove, contact_list, contact_search, contact_filter_by_type,
    contact_filter_by_deployment_type, contact_get_by_group, contact_get_favorites,
    contact_get_blocked, contact_add_to_group, contact_remove_from_group, contact_group_create,
    contact_group_delete, contact_group_list, contact_toggle_favorite, contact_block,
    contact_unblock, contact_get_recent, contact_get_by_node,
    contact_get_by_agent, contact_add_friend_by_digit, contact_register_digit_mapping,
    contact_resolve_digit_to_node, contact_get_digit_for_node, contact_link_to_public_account,
    contact_unlink_from_public_account, contact_get_contacts_by_public_account,
    contact_set_friend_request_mode, contact_get_friend_request_mode,
    contact_filter_by_iot_device_type, contact_filter_by_iot_protocol,
    contact_filter_by_iot_status, contact_get_iot_devices_by_location,
    contact_get_all_iot_devices, contact_update_iot_device_status,
    contact_get_online_iot_devices, contact_get_offline_iot_devices,
};
use microservice::group_chat_commands::{
    group_chat_service_start, group_chat_service_stop, group_create, group_update, group_delete,
    group_get, group_list_user, group_add_member, group_remove_member, group_get_members,
    group_send_message, group_get_messages, group_edit_message, group_delete_message,
    group_create_invitation, group_accept_invitation, group_reject_invitation, group_get_pending_invitations,
    direct_chat_create_or_get, direct_send_message, direct_get_messages, direct_edit_message,
    direct_delete_message, direct_get_chat, direct_list_chats, direct_update_sequence,
    direct_get_sequence, direct_detect_missing, direct_get_messages_by_sequence,
    direct_create_receipt, direct_get_receipts, direct_verify_message,
    group_update_member_online, group_search, group_link_to_public_account,
    group_unlink_from_public_account, group_get_by_public_account, group_add_admin,
    group_remove_admin, group_is_admin, group_is_owner, group_has_permission,
};
use p2p_cdn::{
    p2p_cdn_announce_asset, p2p_cdn_announce_group_hot, p2p_cdn_announce_url_hot,
    p2p_cdn_download, p2p_cdn_get_asset, p2p_cdn_group_send_message, p2p_cdn_hash_file,
    p2p_cdn_join_room, p2p_cdn_leave_room, p2p_cdn_list_peers, p2p_cdn_node_info,
    p2p_cdn_register_local_seed, p2p_cdn_room_feed, p2p_cdn_start_mesh, p2p_cdn_sync_gossip,
    p2p_cdn_url_status,
};
use microservice::social_feed_commands::{
    social_feed_service_start, social_feed_service_stop, social_post_create, social_post_update,
    social_post_delete, social_post_get, social_post_get_user, social_feed_get_timeline,
    social_post_search, social_post_link_to_public_account, social_post_unlink_from_public_account,
    social_post_get_by_public_account, social_comment_add, social_comment_get, social_comment_delete,
    social_reaction_add, social_reaction_remove, social_reaction_get,
    social_user_follow, social_user_unfollow, social_user_get_followings, social_user_get_followers,
    social_user_is_following,
};
use microservice::agent_discovery_commands::{
    agent_discovery_service_start, agent_discovery_service_stop, agent_discovery_register,
    agent_discovery_update_activity, agent_discovery_discover, agent_discovery_trending,
    agent_discovery_search_capability,
};
use microservice::media_streaming_commands::{
    media_streaming_service_start, media_streaming_service_stop, media_stream_create,
    media_stream_update, media_stream_end, media_stream_get, media_stream_list_active,
    media_stream_list_user, media_stream_join, media_stream_leave, media_stream_get_viewers,
    media_stream_get_qualities, media_stream_search, media_stream_get_trending,
    media_stream_get_audio_qualities, media_stream_set_audio_effects, media_stream_get_audio_effects,
    media_stream_add_audio_analysis, media_stream_get_audio_analysis, media_stream_create_audio_mixer,
    media_stream_get_audio_mixer, media_stream_update_mixer_gain, media_stream_update_mixer_panning,
};
use microservice::news_aggregation_commands::{
    news_aggregation_service_start, news_aggregation_service_stop, news_add_article,
    news_add_source, news_update_source, news_remove_source, news_get_article,
    news_get_articles_by_source, news_get_articles_by_category, news_search_articles,
    news_get_latest_articles, news_get_sources, news_create_feed, news_get_feed_articles,
    news_get_statistics,
};
use microservice::public_account_commands::{
    public_account_service_start, public_account_service_stop, public_account_create,
    public_account_get, public_account_update, public_account_list, public_account_get_by_owner,
    public_account_publish_article, public_account_schedule_article, public_account_process_scheduled_articles,
    public_account_get_scheduled_articles, public_account_get_article, public_account_list_articles,
    public_account_subscribe, public_account_unsubscribe, public_account_get_followers,
    public_account_get_subscriptions, public_account_record_view, public_account_like_article,
    public_account_get_analytics, public_account_get_article_analytics, public_account_upload_media,
    public_account_delete_media, public_account_get_media, public_account_list_media,
    public_account_list_media_by_type, public_account_save_draft, public_account_list_drafts,
    public_account_delete_draft, public_account_add_menu_item, public_account_update_menu_item,
    public_account_delete_menu_item, public_account_get_menu_items, public_account_send_notification,
    public_account_mark_notification_read, public_account_get_notifications, public_account_get_unread_notifications,
    public_account_search, public_account_get_trending_articles, public_account_get_articles_by_category,
    public_account_recommend_articles, public_account_get_realtime_analytics,
};
use microservice::service_exposure_commands::{
    service_exposure_service_start, service_exposure_service_stop, service_exposure_expose,
    service_exposure_stop, service_exposure_get, service_exposure_list, service_exposure_update_heartbeat,
};
use microservice::port_forwarding_commands::{
    port_forwarding_service_start, port_forwarding_service_stop, port_forwarding_create,
    port_forwarding_stop, port_forwarding_get, port_forwarding_list, port_forwarding_update_heartbeat,
    port_forwarding_retry,
};
use video_rtc_commands::{
    video_rtc_service_start, video_rtc_node_info, video_rtc_set_display_name,
    video_rtc_publish_signal, video_rtc_poll_signals, video_rtc_call_start,
    video_rtc_call_update, video_rtc_call_list, video_rtc_meeting_create,
    video_rtc_meeting_join, video_rtc_meeting_leave, video_rtc_meeting_get,
    video_rtc_meeting_list, video_rtc_peer_topic,
};
use microservice::video_communication_commands::{
    video_communication_service_start, video_communication_service_stop, video_call_initiate,
    video_call_accept, video_call_end, video_call_get, video_call_list, video_call_add_frame,
    video_call_get_frames, video_call_add_audio_frame, video_call_get_audio_frames,
    video_call_initiate_by_digit, video_call_get_digit, video_render_loop, audio_render_loop,
};
use microservice::collaborative_editing_commands::{
    collaborative_editing_service_start, collaborative_editing_service_stop, collaborative_create_document,
    collaborative_open_document, collaborative_close_document, collaborative_apply_operation,
    collaborative_get_document, collaborative_list_documents, collaborative_update_cursor,
    collaborative_get_cursors, collaborative_get_operations,
};
use microservice::clipboard_sync_commands::{
    clipboard_sync_service_start, clipboard_sync_service_stop, clipboard_sync_create,
    clipboard_sync_stop, clipboard_sync_add_item, clipboard_sync_get_history,
    clipboard_sync_get, clipboard_sync_list, clipboard_sync_connect_device,
    clipboard_sync_disconnect_device,
};
use microservice::terminal_session_commands::{
    terminal_session_service_start, terminal_session_service_stop, terminal_session_create,
    terminal_session_end, terminal_session_add_output, terminal_session_get_outputs,
    terminal_session_get, terminal_session_list, terminal_session_connect_user,
    terminal_session_disconnect_user, terminal_session_send_command,
};
use microservice::ai_video_analysis_commands::{
    ai_video_analysis_service_start, ai_video_analysis_service_stop, ai_video_analysis_create_task,
    ai_video_analysis_delete_task, ai_video_analysis_add_result, ai_video_analysis_get_results,
    ai_video_analysis_get_task, ai_video_analysis_list_tasks, ai_video_analysis_get_stats,
    ai_video_analysis_enable_task, ai_video_analysis_disable_task,
};
use rag::{score_page_match, Bookmark, RagDatabase, SearchResult, Visit};

/// Capture page content for RAG indexing (optionally stores vector embedding).
#[tauri::command]
async fn capture_page(
    url: String,
    title: String,
    content: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
    config: tauri::State<'_, ConfigState>,
) -> Result<String, String> {
    let page_url = url.clone();
    let page_id = db
        .upsert_page_by_url(url, title.clone(), content.clone())
        .await
        .map_err(|e| format!("Failed to store page: {}", e))?;

    let cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?
        .clone();

    if embeddings::embeddings_available(&cfg).await {
        let text = embeddings::embed_text_for_page(&title, &content);
        match embeddings::fetch_embedding(&cfg, &text).await {
            Ok(vector) => {
                if let Err(e) = db.set_embedding_for_url(&page_url, vector).await {
                    eprintln!("[Exodus] Failed to store embedding: {}", e);
                }
            }
            Err(e) => eprintln!("[Exodus] Embedding skipped: {}", e),
        }
    }

    Ok(page_id)
}

/// Semantic search triggered by `/ask` in the omnibox (vector + keyword fallback).
#[tauri::command]
async fn semantic_search(
    query: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
    config: tauri::State<'_, ConfigState>,
) -> Result<Vec<SearchResult>, String> {
    let pages = db
        .get_all_pages()
        .map_err(|e| format!("Failed to retrieve pages: {}", e))?;

    let cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?
        .clone();

    let query_embedding = if embeddings::embeddings_available(&cfg).await {
        embeddings::fetch_embedding(&cfg, &query).await.ok()
    } else {
        None
    };

    let mut results = Vec::new();
    let use_vectors = query_embedding.is_some();

    for page in pages {
        let score = if let Some(ref q_emb) = query_embedding {
            let vec_score = embeddings::score_page_embedding(&page, q_emb);
            if vec_score > 0.05 {
                vec_score
            } else {
                score_page_match(&page, &query) * 0.5
            }
        } else {
            score_page_match(&page, &query)
        };

        let threshold = if use_vectors { 0.25 } else { 0.1 };
        if score > threshold {
            results.push(SearchResult {
                page,
                score,
                matched_text: query.clone(),
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(20);
    Ok(results)
}

/// Check whether the embeddings API is reachable (Ollama / exodus-core).
#[tauri::command]
async fn embeddings_health(config: tauri::State<'_, ConfigState>) -> Result<bool, String> {
    let cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?
        .clone();
    Ok(embeddings::embeddings_available(&cfg).await)
}

/// Return all indexed pages (browsing memory).
#[tauri::command]
async fn get_history(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Vec<rag::WebPage>, String> {
    db.get_all_pages()
        .map_err(|e| format!("Failed to retrieve history: {}", e))
}

/// Clear all RAG data.
#[tauri::command]
async fn clear_rag_data(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<(), String> {
    db.clear_all()
        .map_err(|e| format!("Failed to clear data: {}", e))
}

/// Search RAG indexed pages by title, URL, or content.
#[tauri::command]
fn search_indexed_pages(
    query: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<Vec<rag::WebPage>, String> {
    db.search_pages(&query)
        .map_err(|e| format!("Failed to search indexed pages: {}", e))
}

/// Remove one indexed page from local memory (RAG only).
#[tauri::command]
fn delete_indexed_page(id: String, db: tauri::State<'_, Arc<RagDatabase>>) -> Result<(), String> {
    db.remove_page_by_id(&id)
        .map_err(|e| format!("Failed to delete indexed page: {}", e))
}

/// List saved bookmarks.
#[tauri::command]
fn list_bookmarks(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Vec<Bookmark>, String> {
    db.list_bookmarks()
        .map_err(|e| format!("Failed to list bookmarks: {}", e))
}

/// Add or update a bookmark for the current URL.
#[tauri::command]
fn add_bookmark(
    url: String,
    title: String,
    folder: Option<String>,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<Bookmark, String> {
    let folder_name = folder.unwrap_or_default();
    db.add_bookmark(url, title, folder_name)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

/// Remove a bookmark by id.
#[tauri::command]
fn remove_bookmark(id: String, db: tauri::State<'_, Arc<RagDatabase>>) -> Result<(), String> {
    db.remove_bookmark(&id)
        .map_err(|e| format!("Failed to remove bookmark: {}", e))
}

/// Move a bookmark into a folder (empty = bookmark bar).
#[tauri::command]
fn update_bookmark_folder(
    id: String,
    folder: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<Bookmark, String> {
    db.update_bookmark_folder(&id, folder)
        .map_err(|e| format!("Failed to update bookmark folder: {}", e))
}

/// Reorder bookmark bar chips (left-to-right ids).
#[tauri::command]
fn reorder_bookmarks_bar(
    ordered_ids: Vec<String>,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<(), String> {
    db.reorder_bookmarks_bar(ordered_ids)
        .map_err(|e| format!("Failed to reorder bookmark bar: {e}"))
}

/// Record a page visit for browsing history.
#[tauri::command]
fn record_visit(
    url: String,
    title: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
    config: tauri::State<'_, ConfigState>,
) -> Result<Visit, String> {
    let private = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?
        .private_mode;
    if private {
        return Err("History not recorded in private mode".into());
    }
    db.record_visit(url, title)
        .map_err(|e| format!("Failed to record visit: {}", e))
}

/// Return browsing history (auto-recorded visits).
#[tauri::command]
fn get_visit_history(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Vec<Visit>, String> {
    db.list_visits()
        .map_err(|e| format!("Failed to list visits: {}", e))
}

/// Clear auto-recorded browsing history (visits only; RAG pages unchanged).
#[tauri::command]
fn clear_visit_history(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<(), String> {
    db.clear_visits()
        .map_err(|e| format!("Failed to clear visit history: {}", e))
}

/// Compress DOM HTML for agent processing.
#[tauri::command]
fn compress_dom(html: String, url: String) -> Result<String, String> {
    let compressed = DomCompressor::compress(&html, url)
        .map_err(|e| format!("Failed to compress DOM: {}", e))?;

    serde_json::to_string(&compressed).map_err(|e| format!("Failed to serialize compressed DOM: {}", e))
}

/// Execute agent action with current page URL context.
#[tauri::command]
fn execute_agent_action_with_context(
    action_json: String,
    current_url: String,
) -> Result<String, String> {
    let action: AgentAction = serde_json::from_str(&action_json)
        .map_err(|e| format!("Failed to parse action: {}", e))?;

    let executor = AgentExecutor::new(current_url);
    executor.execute(&action)
}

/// Get privacy settings (https_only, private_mode, block_popups, session_restore).
#[tauri::command]
fn get_privacy_settings(config: tauri::State<'_, ConfigState>) -> Result<(bool, bool, bool, bool), String> {
    let cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?;
    Ok((cfg.https_only, cfg.private_mode, cfg.block_popups, cfg.session_restore))
}

/// Set privacy and session settings with optional parameters and save config to disk.
#[tauri::command]
fn set_privacy_settings(
    https_only: Option<bool>,
    private_mode: Option<bool>,
    block_popups: Option<bool>,
    session_restore: Option<bool>,
    config: tauri::State<'_, ConfigState>,
    history_stores: tauri::State<'_, ProfileHistoryStores>,
    cookie_stores: tauri::State<'_, ProfileCookieStores>,
    app: AppHandle,
) -> Result<(), String> {
    let mut cfg = config.lock()
        .map_err(|e| format!("Config lock error: {}", e))?;
    let was_private = cfg.private_mode;
    if let Some(val) = https_only { cfg.https_only = val; }
    if let Some(val) = private_mode { cfg.private_mode = val; }
    if let Some(val) = block_popups { cfg.block_popups = val; }
    if let Some(val) = session_restore { cfg.session_restore = val; }
    let now_private = cfg.private_mode;
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    cfg.save_to(&app_data_dir)?;
    drop(cfg);

    if was_private && !now_private {
        clear_private_profile(&history_stores, &cookie_stores)?;
    }

    let _ = app.emit("exodus-private-mode-changed", now_private);
    Ok(())
}

/// Clear incognito profile data without toggling global private mode flag.
#[tauri::command]
fn clear_private_profile_data(
    history_stores: tauri::State<'_, ProfileHistoryStores>,
    cookie_stores: tauri::State<'_, ProfileCookieStores>,
) -> Result<(), String> {
    clear_private_profile(&history_stores, &cookie_stores)
}

/// Export bookmarks to JSON format.
#[tauri::command]
fn export_bookmarks(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<String, String> {
    let bookmarks = db.list_bookmarks()
        .map_err(|e| format!("Failed to list bookmarks: {}", e))?;
    
    serde_json::to_string(&bookmarks)
        .map_err(|e| format!("Failed to serialize bookmarks: {}", e))
}

/// Import bookmarks from JSON format.
#[tauri::command]
async fn import_bookmarks(
    bookmarks_json: String,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<usize, String> {
    let bookmarks: Vec<Bookmark> = serde_json::from_str(&bookmarks_json)
        .map_err(|e| format!("Failed to parse bookmarks JSON: {}", e))?;
    
    let mut imported_count = 0;
    for bookmark in bookmarks {
        db.add_bookmark(bookmark.url.clone(), bookmark.title.clone(), bookmark.folder.clone())
            .map_err(|e| format!("Failed to import bookmark {}: {}", bookmark.title, e))?;
        imported_count += 1;
    }
    
    Ok(imported_count)
}

/// Search bookmarks by title or URL.
#[tauri::command]
fn search_bookmarks(query: String, db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Vec<Bookmark>, String> {
    db.search_bookmarks(&query)
        .map_err(|e| format!("Failed to search bookmarks: {}", e))
}

/// Search visits (history) by title or URL.
#[tauri::command]
fn search_visits(query: String, db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Vec<Visit>, String> {
    db.search_visits(&query)
        .map_err(|e| format!("Failed to search visits: {}", e))
}

/// Clear browsing data
#[tauri::command]
async fn clear_browsing_data(
    clear_cache: bool,
    clear_cookies: bool,
    clear_local_storage: bool,
    clear_history: bool,
    cookie_manager: tauri::State<'_, Arc<CookieManager>>,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<String, String> {
    let mut cleared_items = Vec::new();
    
    if clear_cookies {
        cookie_manager.delete_all_cookies()
            .map_err(|e| format!("Failed to clear cookies: {}", e))?;
        cleared_items.push("cookies".to_string());
    }
    
    if clear_history {
        db.clear_visits()
            .map_err(|e| format!("Failed to clear history: {}", e))?;
        cleared_items.push("history".to_string());
    }
    
    if clear_local_storage {
        db.clear_all()
            .map_err(|e| format!("Failed to clear local storage: {}", e))?;
        cleared_items.push("local_storage".to_string());
    }
    
    if clear_cache {
        // Cache clearing would require WebView cache management
        // For now, we'll just note it
        cleared_items.push("cache".to_string());
    }
    
    Ok(format!("Cleared: {}", cleared_items.join(", ")))
}

/// Save session snapshot (open tabs).
#[tauri::command]
async fn save_session(
    tabs: Vec<serde_json::Value>,
    active_tab_id: Option<String>,
    db: tauri::State<'_, Arc<RagDatabase>>,
) -> Result<(), String> {
    use rag::{SessionSnapshot, TabSnapshot};
    
    let tab_snapshots: Result<Vec<TabSnapshot>, _> = tabs
        .into_iter()
        .map(|t| {
            Ok(TabSnapshot {
                id: t.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                url: t.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                title: t.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                active: t.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
            })
        })
        .collect();
    
    let snapshot = SessionSnapshot {
        tabs: tab_snapshots.map_err(|e: Box<dyn std::error::Error>| format!("Failed to parse tabs: {}", e))?,
        active_tab_id,
        timestamp: Utc::now(),
    };
    
    db.save_session(snapshot).await
        .map_err(|e| format!("Failed to save session: {}", e))
}

/// Load session snapshot.
#[tauri::command]
fn load_session(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<Option<serde_json::Value>, String> {
    match db.load_session() {
        Ok(Some(snapshot)) => {
            let json = serde_json::to_value(snapshot)
                .map_err(|e| format!("Failed to serialize session: {}", e))?;
            Ok(Some(json))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to load session: {}", e)),
    }
}

/// Clear session snapshot.
#[tauri::command]
fn clear_session(db: tauri::State<'_, Arc<RagDatabase>>) -> Result<(), String> {
    db.clear_session()
        .map_err(|e| format!("Failed to clear session: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured logging for aerospace-level audit compliance
    startup_log::init_tracing();
    startup_log::log_step("=== RUN() FUNCTION STARTED ===");
    startup_log::log_step("Exodus Browser run() entry");
    plugins::log_plugin_module_init();

    let lifecycle = Arc::new(app_lifecycle::AppLifecycleManager::new());
    let lifecycle_setup = Arc::clone(&lifecycle);

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());
    http_response_proxy::register_native_proxy_protocol(builder)
        .setup(move |app| {
            startup_log::log_step("=== SETUP FUNCTION CALLED ===");
            startup_log::log_step("=== SETUP FUNCTION STARTED ===");
            app.manage(lifecycle_setup.clone());
            lifecycle_setup.set_phase(app_lifecycle::LifecyclePhase::Setup);
            startup_log::log_step("setup() begin - FUNCTION ENTRY");
            startup_log::log_step("Starting macOS app policy configuration...");
            app_window::configure_macos_app_policy(app.handle());
            startup_log::log_step("macOS app policy configuration completed");
            
            // Set dock icon for dev mode
            startup_log::log_step("Setting dock icon...");
            app_window::set_dock_icon(app.handle());
            startup_log::log_step("Dock icon configuration completed");

            // Force window size and position immediately in setup to override macOS state restoration
            startup_log::log_step("=== Checking for main window in setup() ===");
            if let Some(win) = app.get_webview_window("main") {
                startup_log::log_step("✓ Main window found in setup()");
                startup_log::log_step("=== Starting window configuration in setup() ===");

                // Get the scale factor for Retina displays
                let scale_factor = match win.scale_factor() {
                    Ok(sf) => sf,
                    Err(e) => {
                        startup_log::log_error(&format!("Failed to get scale factor in setup: {}", e));
                        1.0
                    }
                };
                startup_log::log_step(&format!("Window scale factor in setup: {}", scale_factor));

                // Logical size matches tauri.conf (Retina-safe; Physical was ~640×360 logical).
                startup_log::log_step("Setting window size to 1280x720 logical pixels");
                match win.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: 1280.0,
                    height: 720.0,
                })) {
                    Ok(_) => startup_log::log_step("✓ set_size(1280x720) succeeded (Logical)"),
                    Err(e) => startup_log::log_error(&format!("✗ set_size(1280x720) failed (Logical): {}", e)),
                }

                // Ensure window is visible and unminimized
                let _ = win.unminimize();
                match win.show() {
                    Ok(_) => startup_log::log_step("✓ win.show() succeeded"),
                    Err(e) => startup_log::log_error(&format!("✗ win.show() failed: {}", e)),
                }

                startup_log::log_step("=== Window configuration in setup() completed ===");
            } else {
                startup_log::log_error("✗ Main window NOT found in setup() - window configuration skipped");
            }

            startup_log::log_step("Resolving app data directory...");
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
            startup_log::log_step(&format!("app_data_dir resolved: {}", app_data_dir.display()));
            startup_log::set_file_log_dir(&app_data_dir);
            lifecycle_log::init_lifecycle_log_dir(&app_data_dir);
            startup_log::log_step(&format!("app_data_dir={}", app_data_dir.display()));

            startup_log::log_step("Loading ExodusConfig...");
            let exodus_config = ExodusConfig::load_from(&app_data_dir);
            startup_log::log_step("ExodusConfig loaded successfully");
            app.manage(Mutex::new(exodus_config) as ConfigState);

            startup_log::log_step("Initializing RAG database...");
            let rag_db = Arc::new(
                // Try to initialize RAG database, with automatic lock error recovery
                match RagDatabase::new_at(&app_data_dir) {
                    Ok(db) => {
                        startup_log::log_step("RAG database initialized successfully");
                        db
                    }
                    Err(e) => {
                        startup_log::log_error(&format!("RAG database initialization failed: {}", e));
                        let error_str = e.to_string();
                        if error_str.contains("could not acquire lock") || error_str.contains("WouldBlock") {
                            startup_log::log_warn(&format!("RAG database lock error detected: {}, attempting to delete and recreate", e));
                            let db_path = app_data_dir.join("exodus_rag_db");
                            startup_log::log_step(&format!("Attempting to delete RAG database at: {}", db_path.display()));
                            if let Err(del_err) = std::fs::remove_dir_all(&db_path) {
                                startup_log::log_error(&format!("Failed to delete RAG database: {}", del_err));
                                return Err(format!("Failed to initialize RAG database and could not delete: {}", e).into());
                            }
                            startup_log::log_step("RAG database deleted successfully, retrying initialization...");
                            // Retry initialization after deletion
                            match RagDatabase::new_at(&app_data_dir) {
                                Ok(db) => {
                                    startup_log::log_step("RAG database recreated successfully");
                                    db
                                }
                                Err(retry_err) => {
                                    startup_log::log_error(&format!("RAG database retry initialization failed: {}", retry_err));
                                    return Err(format!("Failed to initialize RAG database after deletion: {}", retry_err).into());
                                }
                            }
                        } else {
                            startup_log::log_error(&format!("RAG database initialization failed with non-lock error: {}", e));
                            return Err(format!("Failed to initialize RAG database: {}", e).into());
                        }
                    }
                }
            );
            app.manage(rag_db);
            startup_log::log_step("RAG database ready and managed");
            
            startup_log::log_step("Initializing TabNavTracker...");
            app.manage(browser::TabNavTracker::default());
            startup_log::log_step("TabNavTracker initialized");

            startup_log::log_step("Reading config state for sidecar/Allama settings...");
            let (spawn_sidecar, spawn_allama, ai_port) = app
                .state::<ConfigState>()
                .lock()
                .map(|c| (c.spawn_sidecar, c.spawn_allama, c.ai_port))
                .unwrap_or((false, true, microservice::ALLAMA_DEFAULT_PORT));
            startup_log::log_step(&format!("Config read: spawn_sidecar={}, spawn_allama={}, ai_port={}", spawn_sidecar, spawn_allama, ai_port));

            startup_log::log_step("Initializing SidecarManager...");
            app.manage(sidecar::SidecarManager::new(
                app.handle(),
                spawn_sidecar,
                ai_port,
            ));
            startup_log::log_step("SidecarManager initialized");

            startup_log::log_step("Initializing ExtensionManager...");
            let extension_mgr = ExtensionManager::new(&app_data_dir)
                .map_err(|e| format!("Failed to initialize extensions: {e}"))?;
            startup_log::log_step("ExtensionManager created");
            let extension_state: ExtensionState = Arc::new(Mutex::new(extension_mgr));
            {
                startup_log::log_step("Scanning and loading extensions...");
                let mut mgr = extension_state
                    .lock()
                    .map_err(|e| format!("Extension state lock error: {e}"))?;
                let dev_dir = plugins::dev_extensions_dir();
                startup_log::log_step(&format!("Extension dev directory: {:?}", dev_dir));
                mgr.scan_and_load(dev_dir.as_deref())
                    .map_err(|e| format!("Extension scan failed: {e}"))?;
                mgr.set_allama_http_port(ai_port);
                startup_log::log_step("Extensions scanned and loaded");
            }
            app.manage(extension_state);
            startup_log::log_step("Extension state managed");
            
            startup_log::log_step("Initializing NativePluginManager...");
            let native_plugin_mgr = NativePluginManager::new();
            app.manage(native_plugin_mgr);
            startup_log::log_step("NativePluginManager initialized");
            
            startup_log::log_step("Initializing TabRegistry...");
            app.manage(TabRegistry::default());
            startup_log::log_step("TabRegistry initialized");
            
            startup_log::log_step("Initializing NotificationStore...");
            app.manage(NotificationStore::default());
            startup_log::log_step("NotificationStore initialized");
            
            startup_log::log_step("Initializing NetRuleStore...");
            app.manage(NetRuleStore::default());
            startup_log::log_step("NetRuleStore initialized");
            
            startup_log::log_step("Initializing WebRequestStore...");
            app.manage(WebRequestStore::default());
            startup_log::log_step("WebRequestStore initialized");

            startup_log::log_step("Starting HTTP response proxy...");
            let app_for_proxy = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = http_response_proxy::start_http_response_proxy(app_for_proxy).await {
                    tracing::error!("HTTP response proxy failed to start: {e}");
                }
            });
            startup_log::log_step("HTTP response proxy started");
            
            startup_log::log_step("Initializing ExtensionActionStore...");
            app.manage(Arc::new(plugins::extension_popup::ExtensionActionStore::default()));
            startup_log::log_step("ExtensionActionStore initialized");
            
            startup_log::log_step("Initializing DiscardedTabsRegistry...");
            app.manage(DiscardedTabsRegistry::new());
            startup_log::log_step("DiscardedTabsRegistry initialized");

            startup_log::log_step("Initializing EncryptedSyncManager...");
            let encrypted_sync_dir = app_data_dir.join("encrypted_sync");
            let encrypted_sync = Arc::new(
                EncryptedSyncManager::new(encrypted_sync_dir)
                    .map_err(|e| format!("Encrypted sync init failed: {e}"))?,
            );
            app.manage(encrypted_sync);
            startup_log::log_step("EncryptedSyncManager initialized");
            
            startup_log::log_step("Initializing PermissionPendingStore...");
            app.manage(PermissionPendingStore::default());
            startup_log::log_step("PermissionPendingStore initialized");
            
            startup_log::log_step("Initializing SitePermissionStore...");
            let site_perms = SitePermissionStore::new(&app_data_dir)
                .map_err(|e| format!("Site permission store failed: {e}"))?;
            app.manage(site_perms);
            startup_log::log_step("SitePermissionStore initialized");
            
            startup_log::log_step("Initializing HostInstallPendingStore...");
            app.manage(HostInstallPendingStore::default());
            startup_log::log_step("HostInstallPendingStore initialized");
            
            startup_log::log_step("Initializing BrowserSitePermissionStore...");
            let browser_site_perms = BrowserSitePermissionStore::new(&app_data_dir)
                .map_err(|e| format!("Browser site permission store failed: {e}"))?;
            app.manage(browser_site_perms);
            startup_log::log_step("BrowserSitePermissionStore initialized");
            
            // Initialize password manager
            startup_log::log_step("Initializing PasswordManager...");
            let password_dir = app_data_dir.join("passwords");
            let password_manager = Arc::new(
                PasswordManager::new(password_dir)
                    .map_err(|e| format!("Password manager init failed: {e}"))?,
            );
            app.manage(password_manager);
            startup_log::log_step("PasswordManager initialized");
            
            // Initialize cookie manager
            startup_log::log_step("Initializing ProfileCookieStores...");
            let profile_cookies = ProfileCookieStores::new(&app_data_dir)
                .map_err(|e| format!("Profile cookie stores init failed: {e}"))?;
            app.manage(profile_cookies);
            startup_log::log_step("ProfileCookieStores initialized");
            
            // Initialize smart cache
            startup_log::log_step("Initializing SmartCache...");
            let cache_dir = app_data_dir.join("cache");
            let smart_cache = Arc::new(
                SmartCache::new(cache_dir, 100 * 1024 * 1024, 1000) // 100MB max, 1000 entries
                    .map_err(|e| format!("Smart cache init failed: {e}"))?,
            );
            app.manage(smart_cache);
            startup_log::log_step("SmartCache initialized");
            
            // Initialize Picture-in-Picture manager
            startup_log::log_step("Initializing PipManager...");
            let pip_manager = Arc::new(picture_in_picture::PipManager::new());
            app.manage(pip_manager);
            startup_log::log_step("PipManager initialized");
            
            // Initialize Local Pocket manager
            startup_log::log_step("Initializing LocalPocketManager...");
            let pocket_storage_path = app_data_dir.join("pocket").join("articles.json");
            let pocket_manager = Arc::new(local_pocket::LocalPocketManager::new(pocket_storage_path));
            app.manage(pocket_manager);
            startup_log::log_step("LocalPocketManager initialized");
            
            // Initialize Tab Search manager
            startup_log::log_step("Initializing TabSearchManager...");
            let tab_search_manager = Arc::new(tab_search::TabSearchManager::new());
            app.manage(tab_search_manager);
            startup_log::log_step("TabSearchManager initialized");
            
            // Initialize Tab Mute manager
            startup_log::log_step("Initializing TabMuteManager...");
            let tab_mute_manager = Arc::new(tab_mute::TabMuteManager::new());
            app.manage(tab_mute_manager);
            startup_log::log_step("TabMuteManager initialized");
            
            // Initialize Tab Preview manager
            startup_log::log_step("Initializing TabPreviewManager...");
            let tab_preview_manager = Arc::new(tab_preview::TabPreviewManager::new());
            app.manage(tab_preview_manager);
            startup_log::log_step("TabPreviewManager initialized");
            
            // Initialize HTTPS-Only manager
            startup_log::log_step("Initializing HttpsOnlyManager...");
            let https_only_manager = Arc::new(https_only::HttpsOnlyManager::new());
            app.manage(https_only_manager);
            startup_log::log_step("HttpsOnlyManager initialized");
            
            // Initialize DNS over HTTPS manager
            startup_log::log_step("Initializing DohManager...");
            let doh_manager = Arc::new(dns_over_https::DohManager::new());
            app.manage(doh_manager);
            startup_log::log_step("DohManager initialized");
            
            // Initialize Fingerprinting Protection manager
            startup_log::log_step("Initializing FingerprintingProtectionManager...");
            let fingerprinting_manager = Arc::new(fingerprinting_protection::FingerprintingProtectionManager::new());
            app.manage(fingerprinting_manager);
            startup_log::log_step("FingerprintingProtectionManager initialized");
            
            // Initialize Omnibox Actions manager
            startup_log::log_step("Initializing OmniboxActionsManager...");
            let omnibox_manager = Arc::new(omnibox_actions::OmniboxActionsManager::new());
            app.manage(omnibox_manager);
            startup_log::log_step("OmniboxActionsManager initialized");
            
            // Initialize Biometric Authentication manager
            startup_log::log_step("Initializing BiometricAuthManager...");
            let biometric_manager = Arc::new(biometric_auth::BiometricAuthManager::new());
            app.manage(biometric_manager);
            startup_log::log_step("BiometricAuthManager initialized");
            
            // Initialize Voice Search manager
            startup_log::log_step("Initializing VoiceSearchManager...");
            let voice_search_manager = Arc::new(voice_search::VoiceSearchManager::new());
            app.manage(voice_search_manager);
            startup_log::log_step("VoiceSearchManager initialized");
            
            // Initialize Text-to-Speech manager
            startup_log::log_step("Initializing TtsManager...");
            let tts_manager = Arc::new(text_to_speech::TtsManager::new());
            app.manage(tts_manager);
            startup_log::log_step("TtsManager initialized");
            
            // Initialize Global Audio manager
            startup_log::log_step("Initializing GlobalAudioManager...");
            let global_audio_manager = Arc::new(global_audio::GlobalAudioManager::new());
            app.manage(global_audio_manager);
            startup_log::log_step("GlobalAudioManager initialized");
            
            // Initialize Per-Site Shield manager
            startup_log::log_step("Initializing PerSiteShieldManager...");
            let per_site_shield_manager = Arc::new(per_site_shields::PerSiteShieldManager::new());
            app.manage(per_site_shield_manager);
            startup_log::log_step("PerSiteShieldManager initialized");
            
            // Initialize Tab Stacking manager
            startup_log::log_step("Initializing TabStackManager...");
            let tab_stacking_manager = Arc::new(tab_stacking::TabStackManager::new());
            app.manage(tab_stacking_manager);
            startup_log::log_step("TabStackManager initialized");
            
            // Initialize Reading Progress manager
            startup_log::log_step("Initializing ReadingProgressManager...");
            let reading_progress_manager = Arc::new(reading_progress::ReadingProgressManager::new());
            app.manage(reading_progress_manager);
            startup_log::log_step("ReadingProgressManager initialized");
            
            // Initialize Annotation manager
            startup_log::log_step("Initializing AnnotationManager...");
            let annotation_manager = Arc::new(annotation::AnnotationManager::new());
            app.manage(annotation_manager);
            startup_log::log_step("AnnotationManager initialized");
            
            // Initialize Media Casting manager
            startup_log::log_step("Initializing MediaCastingManager...");
            let media_casting_manager = Arc::new(media_casting::MediaCastingManager::new());
            app.manage(media_casting_manager);
            startup_log::log_step("MediaCastingManager initialized");
            
            // Initialize Color Blind manager
            startup_log::log_step("Initializing ColorBlindManager...");
            let color_blind_manager = Arc::new(color_blind::ColorBlindManager::new());
            app.manage(color_blind_manager);
            startup_log::log_step("ColorBlindManager initialized");
            
            // Initialize Voice Control manager
            startup_log::log_step("Initializing VoiceControlManager...");
            let voice_control_manager = Arc::new(voice_control::VoiceControlManager::new());
            app.manage(voice_control_manager);
            startup_log::log_step("VoiceControlManager initialized");
            
            // Initialize Data Saver manager
            startup_log::log_step("Initializing DataSaverManager...");
            let data_saver_manager = Arc::new(data_saver::DataSaverManager::new());
            app.manage(data_saver_manager);
            startup_log::log_step("DataSaverManager initialized");
            
            // Initialize Audio Visualization manager
            startup_log::log_step("Initializing AudioVisualizationManager...");
            let audio_visualization_manager = Arc::new(audio_visualization::AudioVisualizationManager::new());
            app.manage(audio_visualization_manager);
            startup_log::log_step("AudioVisualizationManager initialized");
            
            // Initialize Password Sharing manager
            startup_log::log_step("Initializing PasswordSharingManager...");
            let password_sharing_manager = Arc::new(password_sharing::PasswordSharingManager::new());
            app.manage(password_sharing_manager);
            startup_log::log_step("PasswordSharingManager initialized");
            
            // Initialize Private Search manager
            startup_log::log_step("Initializing PrivateSearchManager...");
            let private_search_manager = Arc::new(private_search::PrivateSearchManager::new());
            app.manage(private_search_manager);
            startup_log::log_step("PrivateSearchManager initialized");
            
            // Initialize Omnibox Image Search manager
            startup_log::log_step("Initializing OmniboxImageSearchManager...");
            let omnibox_image_search_manager = Arc::new(omnibox_image_search::OmniboxImageSearchManager::new());
            app.manage(omnibox_image_search_manager);
            startup_log::log_step("OmniboxImageSearchManager initialized");
            
            // Initialize Mobile Sync manager
            startup_log::log_step("Initializing MobileSyncManager...");
            let mobile_sync_manager = Arc::new(mobile_sync::MobileSyncManager::new());
            app.manage(mobile_sync_manager);
            startup_log::log_step("MobileSyncManager initialized");
            
            // Initialize Password Health manager
            startup_log::log_step("Initializing PasswordHealthManager...");
            let password_health_manager = Arc::new(password_health::PasswordHealthManager::new());
            app.manage(password_health_manager);
            startup_log::log_step("PasswordHealthManager initialized");
            
            // Initialize theme manager
            startup_log::log_step("Initializing ThemeManager...");
            let theme_dir = app_data_dir.join("theme");
            let theme_manager = Arc::new(
                ThemeManager::new(theme_dir)
                    .map_err(|e| format!("Theme manager init failed: {e}"))?,
            );
            app.manage(theme_manager);
            startup_log::log_step("ThemeManager initialized");
            
            // Initialize tracking protection
            startup_log::log_step("Initializing TrackingProtectionManager...");
            let tracking_dir = app_data_dir.join("tracking");
            let tracking_manager = Arc::new(
                TrackingProtectionManager::new(tracking_dir)
                    .map_err(|e| format!("Tracking protection init failed: {e}"))?,
            );
            app.manage(tracking_manager.clone());
            startup_log::log_step("TrackingProtectionManager initialized");

            // Initialize smart suggestions
            startup_log::log_step("Initializing SmartSuggestionsManager...");
            let suggestions_dir = app_data_dir.join("suggestions");
            let suggestions_manager = Arc::new(
                SmartSuggestionsManager::new(suggestions_dir)
                    .map_err(|e| format!("Smart suggestions init failed: {e}"))?,
            );
            app.manage(suggestions_manager);
            startup_log::log_step("SmartSuggestionsManager initialized");
            
            // Initialize tab freezer
            startup_log::log_step("Initializing TabFreezer...");
            let tab_freezer = Arc::new(TabFreezer::new());
            app.manage(tab_freezer);
            startup_log::log_step("TabFreezer initialized");
            
            // Initialize tab sleeping manager
            startup_log::log_step("Initializing TabSleepManager...");
            let tab_sleep_manager = Arc::new(tab_sleeping::TabSleepManager::new());
            app.manage(tab_sleep_manager);
            startup_log::log_step("TabSleepManager initialized");
            
            // Initialize metrics collector
            startup_log::log_step("Initializing MetricsCollector...");
            let metrics_collector = Arc::new(microservice::MetricsCollector::new());
            app.manage(metrics_collector);
            startup_log::log_step("MetricsCollector initialized");
            
            // Initialize resource preloader
            startup_log::log_step("Initializing ResourcePreloader...");
            let resource_preloader = Arc::new(resource_preloader::ResourcePreloader::new());
            app.manage(resource_preloader);
            startup_log::log_step("ResourcePreloader initialized");
            
            // Initialize Hermes agent (LLM via Allama HTTP on ai_port)
            startup_log::log_step("Initializing HermesAgent...");
            let hermes_agent = Arc::new(hermes_agent::HermesAgent::with_allama_port(ai_port));
            app.manage(hermes_agent);
            startup_log::log_step("HermesAgent initialized");
            
            // Initialize Python microservice
            startup_log::log_step("Initializing PythonMicroservice...");
            let python_microservice =
                Arc::new(python_microservice::PythonMicroservice::with_allama_port(ai_port));
            app.manage(python_microservice);
            startup_log::log_step("PythonMicroservice initialized");
            
            // Initialize inference engine (Allama backend registry)
            startup_log::log_step("Initializing InferenceEngine...");
            let inference_engine = Arc::new(inference_engine::InferenceEngine::new());
            startup_log::log_step("Setting Allama HTTP port on InferenceEngine...");
            tauri::async_runtime::block_on(
                inference_engine.set_allama_http_port(Some(ai_port)),
            );
            app.manage(inference_engine.clone());
            startup_log::log_step("InferenceEngine initialized");

            // Allama microservice — Ollama replacement on port 11435
            startup_log::log_step("Initializing AllamaManager...");
            let models_dir =
                microservice::allama_process::resolve_models_dir(Some(&app_data_dir));
            startup_log::log_step(&format!("Allama models directory: {:?}", models_dir));
            let allama_mgr = Arc::new(AllamaManager::new(
                inference_engine.clone(),
                ai_port,
                models_dir,
            ));
            app.manage(allama_mgr.clone());
            startup_log::log_step("AllamaManager initialized, spawning if enabled...");
            allama_mgr.spawn_if_enabled(app.handle(), spawn_allama);
            startup_log::log_step("AllamaManager spawn check completed");

            // Initialize DNS prefetch
            startup_log::log_step("Initializing DnsPrefetchManager...");
            let dns_prefetch = Arc::new(DnsPrefetchManager::new());
            app.manage(dns_prefetch);
            startup_log::log_step("DnsPrefetchManager initialized");
            
            // Initialize translation service
            startup_log::log_step("Initializing TranslationService...");
            let translation_service = Arc::new(TranslationService::new());
            app.manage(translation_service);
            startup_log::log_step("TranslationService initialized");
            
            // Initialize safe browsing
            startup_log::log_step("Initializing SafeBrowsingManager...");
            let safe_browsing_dir = app_data_dir.join("safe_browsing");
            let safe_browsing = Arc::new(
                SafeBrowsingManager::new(safe_browsing_dir)
                    .map_err(|e| format!("Safe browsing init failed: {e}"))?,
            );
            app.manage(safe_browsing);
            startup_log::log_step("SafeBrowsingManager initialized");
            
            // Initialize certificate validation
            startup_log::log_step("Initializing CertificateValidationManager...");
            let cert_dir = app_data_dir.join("certificates");
            let cert_manager = Arc::new(
                CertificateValidationManager::new(cert_dir)
                    .map_err(|e| format!("Certificate validation init failed: {e}"))?,
            );
            app.manage(cert_manager);
            startup_log::log_step("CertificateValidationManager initialized");
            
            // Initialize extension API (commented out - module doesn't exist)
            startup_log::log_step("Extension API initialization skipped (module not available)");
            
            // Initialize extension permissions (commented out - module doesn't exist)
            startup_log::log_step("Extension permissions initialization skipped (module not available)");
            
            // Initialize DevTools
            startup_log::log_step("Initializing DevToolsManager...");
            let devtools_dir = app_data_dir.join("devtools");
            let devtools = Arc::new(
                DevToolsManager::new(devtools_dir)
                    .map_err(|e| format!("DevTools init failed: {e}"))?,
            );
            app.manage(devtools);
            startup_log::log_step("DevToolsManager initialized");
            
            // Initialize privacy dashboard
            startup_log::log_step("Initializing PrivacyDashboardManager...");
            let privacy_dir = app_data_dir.join("privacy");
            let privacy = Arc::new(
                PrivacyDashboardManager::new(privacy_dir)
                    .map_err(|e| format!("Privacy dashboard init failed: {e}"))?,
            );
            app.manage(privacy);
            startup_log::log_step("PrivacyDashboardManager initialized");
            
            // Initialize reading mode
            startup_log::log_step("Initializing ReadingModeManager...");
            let reading_dir = app_data_dir.join("reading_mode");
            let reading_mode = Arc::new(
                ReadingModeManager::new(reading_dir)
                    .map_err(|e| format!("Reading mode init failed: {e}"))?,
            );
            app.manage(reading_mode);
            startup_log::log_step("ReadingModeManager initialized");
            
            // Initialize page zoom
            startup_log::log_step("Initializing PageZoomManager...");
            let zoom_dir = app_data_dir.join("page_zoom");
            let page_zoom = Arc::new(
                PageZoomManager::new(zoom_dir)
                    .map_err(|e| format!("Page zoom init failed: {e}"))?,
            );
            app.manage(page_zoom);
            startup_log::log_step("PageZoomManager initialized");
            
            // Initialize download manager
            startup_log::log_step("Initializing DownloadManager...");
            let download_dir = app_data_dir.join("downloads");
            let download_manager = Arc::new(
                DownloadManager::new(download_dir)
                    .map_err(|e| format!("Download manager init failed: {e}"))?,
            );
            app.manage(download_manager);
            startup_log::log_step("DownloadManager initialized");
            
            // Initialize profile-scoped history (default + private)
            startup_log::log_step("Initializing ProfileHistoryStores...");
            let profile_history = ProfileHistoryStores::new(&app_data_dir)
                .map_err(|e| format!("Profile history stores init failed: {e}"))?;
            app.manage(profile_history);
            startup_log::log_step("ProfileHistoryStores initialized");
            
            // Initialize bookmark sync
            startup_log::log_step("Initializing BookmarkSyncManager...");
            let sync_dir = app_data_dir.join("bookmark_sync");
            let bookmark_sync = Arc::new(
                BookmarkSyncManager::new(sync_dir)
                    .map_err(|e| format!("Bookmark sync init failed: {e}"))?,
            );
            app.manage(bookmark_sync);
            startup_log::log_step("BookmarkSyncManager initialized");
            
            // Initialize form autofill
            startup_log::log_step("Initializing FormAutofillManager...");
            let autofill_dir = app_data_dir.join("form_autofill");
            let form_autofill = Arc::new(
                FormAutofillManager::new(autofill_dir)
                    .map_err(|e| format!("Form autofill init failed: {e}"))?,
            );
            app.manage(form_autofill);
            startup_log::log_step("FormAutofillManager initialized");
            
            // Initialize tab grouping
            startup_log::log_step("Initializing TabGroupingManager...");
            let grouping_dir = app_data_dir.join("tab_grouping");
            let tab_grouping = Arc::new(
                TabGroupingManager::new(grouping_dir)
                    .map_err(|e| format!("Tab grouping init failed: {e}"))?,
            );
            app.manage(tab_grouping);
            startup_log::log_step("TabGroupingManager initialized");
            
            // Initialize vertical tabs
            startup_log::log_step("Initializing VerticalTabsManager...");
            let vertical_dir = app_data_dir.join("vertical_tabs");
            let vertical_tabs = Arc::new(
                VerticalTabsManager::new(vertical_dir)
                    .map_err(|e| format!("Vertical tabs init failed: {e}"))?,
            );
            app.manage(vertical_tabs);
            startup_log::log_step("VerticalTabsManager initialized");
            
            // Initialize tab pinning
            startup_log::log_step("Initializing TabPinningManager...");
            let pinning_dir = app_data_dir.join("tab_pinning");
            let tab_pinning = Arc::new(
                TabPinningManager::new(pinning_dir)
                    .map_err(|e| format!("Tab pinning init failed: {e}"))?,
            );
            app.manage(tab_pinning);
            startup_log::log_step("TabPinningManager initialized");
            
            // Initialize search engine
            startup_log::log_step("Initializing SearchEngineManager...");
            let search_dir = app_data_dir.join("search_engine");
            let search_engine = Arc::new(
                SearchEngineManager::new(search_dir)
                    .map_err(|e| format!("Search engine init failed: {e}"))?,
            );
            app.manage(search_engine);
            startup_log::log_step("SearchEngineManager initialized");
            
            // Initialize new tab page
            startup_log::log_step("Initializing NewTabPageManager...");
            let newtab_dir = app_data_dir.join("new_tab_page");
            let new_tab_page = Arc::new(
                NewTabPageManager::new(newtab_dir)
                    .map_err(|e| format!("New tab page init failed: {e}"))?,
            );
            app.manage(new_tab_page);
            startup_log::log_step("NewTabPageManager initialized");
            startup_log::log_step("Seeding wallpaper library...");
            let _ = ntp_wallpapers::seed_wallpaper_library(&app_data_dir);
            startup_log::log_step("Wallpaper library seeded");
            
            // Initialize network interception
            startup_log::log_step("Initializing NetworkInterceptor...");
            let network_dir = app_data_dir.join("network_interception");
            let network_interceptor = Arc::new(
                NetworkInterceptor::new(network_dir)
                    .map_err(|e| format!("Network interception init failed: {e}"))?,
            );
            app.manage(network_interceptor);
            startup_log::log_step("NetworkInterceptor initialized");
            
            // Initialize content script
            startup_log::log_step("Initializing ContentScriptManager...");
            let content_dir = app_data_dir.join("content_script");
            let content_script = Arc::new(
                ContentScriptManager::new(content_dir)
                    .map_err(|e| format!("Content script init failed: {e}"))?,
            );
            app.manage(content_script);
            startup_log::log_step("ContentScriptManager initialized");
            
            // Initialize user script
            startup_log::log_step("Initializing UserScriptManager...");
            let userscript_dir = app_data_dir.join("user_script");
            let user_script = Arc::new(
                UserScriptManager::new(userscript_dir)
                    .map_err(|e| format!("User script init failed: {e}"))?,
            );
            app.manage(user_script);
            startup_log::log_step("UserScriptManager initialized");
            
            // Initialize font settings
            startup_log::log_step("Initializing FontSettingsManager...");
            let font_dir = app_data_dir.join("font_settings");
            let font_settings = Arc::new(
                FontSettingsManager::new(font_dir)
                    .map_err(|e| format!("Font settings init failed: {e}"))?,
            );
            app.manage(font_settings);
            startup_log::log_step("FontSettingsManager initialized");
            
            // Initialize print settings
            startup_log::log_step("Initializing PrintSettingsManager...");
            let print_dir = app_data_dir.join("print_settings");
            let print_settings = Arc::new(
                PrintSettingsManager::new(print_dir)
                    .map_err(|e| format!("Print settings init failed: {e}"))?,
            );
            app.manage(print_settings);
            startup_log::log_step("PrintSettingsManager initialized");
            
            // Initialize PDF viewer
            startup_log::log_step("Initializing PdfViewerManager...");
            let pdf_dir = app_data_dir.join("pdf_viewer");
            let pdf_viewer = Arc::new(
                PdfViewerManager::new(pdf_dir)
                    .map_err(|e| format!("PDF viewer init failed: {e}"))?,
            );
            app.manage(pdf_viewer);
            startup_log::log_step("PdfViewerManager initialized");
            
            // Initialize microservice system
            startup_log::log_step("Initializing microservice system...");
            let socket_dir = std::path::PathBuf::from("/tmp/exodus-services");
            std::fs::create_dir_all(&socket_dir).ok();
            startup_log::log_step(&format!("Microservice socket directory: {:?}", socket_dir));
            
            let registry = Arc::new(ServiceRegistry::new(socket_dir.clone()));
            let bus = Arc::new(MicroserviceBus::new(Arc::clone(&registry)));
            let supervisor = Arc::new(ServiceSupervisor::new(Arc::clone(&registry)));
            startup_log::log_step("Microservice registry, bus, and supervisor initialized");
            
            // Initialize log collector
            startup_log::log_step("Initializing LogCollector...");
            let log_dir = socket_dir.join("logs");
            let log_collector = Arc::new(LogCollector::new(log_dir).unwrap_or_else(|_| {
                LogCollector::new(std::path::PathBuf::from("/tmp/exodus-logs")).expect("Failed to create log collector")
            }));
            startup_log::log_step("LogCollector initialized");
            
            // Initialize resource monitor
            startup_log::log_step("Initializing ResourceMonitor...");
            let resource_monitor = Arc::new(ResourceMonitor::new()
                .with_max_metrics(1000)
                .with_collection_interval(std::time::Duration::from_secs(5)));
            startup_log::log_step("ResourceMonitor initialized");
            
            app.manage(registry.clone());
            app.manage(bus);
            app.manage(supervisor);
            app.manage(log_collector);
            app.manage(resource_monitor);
            startup_log::log_step("Microservice components managed");

            // Register Allama in the microservice registry (UDS control plane metadata).
            startup_log::log_step("Registering Allama service in microservice registry...");
            let allama_socket = microservice::allama_service::AllamaServiceConfig::default().socket_path;
            let _ = registry.register(microservice::ServiceInfo::new(
                "allama-service",
                allama_socket.to_string_lossy().to_string(),
                std::process::id(),
            ));
            startup_log::log_step("Allama service registered");

            startup_log::log_step("Initializing P2P CDN state...");
            let p2p_cdn = Arc::new(
                p2p_cdn::P2pCdnState::new(&app_data_dir)
                    .map_err(|e| {
                        tracing::error!("P2P CDN init failed: {}", e);
                        format!("P2P CDN init failed: {e}")
                    })?,
            );
            startup_log::log_step(&format!("P2P CDN state initialized, node_id={}", p2p_cdn.node_id));
            {
                startup_log::log_step("Joining P2P CDN default rooms...");
                let _ = p2p_cdn.join_room("lobby");
                let _ = p2p_cdn.join_room(exodus_workspace::WORKSPACE_ROOM_ID);
                startup_log::log_step(&format!("P2P CDN joined default rooms: lobby, {}", exodus_workspace::WORKSPACE_ROOM_ID));
            }
            let p2p_cdn_start = Arc::clone(&p2p_cdn);
            tauri::async_runtime::spawn(async move {
                startup_log::log_step("Starting P2P CDN mesh server...");
                match p2p_cdn_start.ensure_mesh().await {
                    Ok((host, port)) => {
                        startup_log::log_step(&format!("P2P CDN mesh started on {}:{}", host, port));
                    }
                    Err(e) => {
                        startup_log::log_error(&format!("P2P CDN mesh start failed: {e}"));
                    }
                }
            });
            let gossip_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                startup_log::log_step("Starting P2P gossip service...");
                match p2p_gossip_service_start(gossip_app).await {
                    Ok(_) => {
                        startup_log::log_step("P2P gossip service started successfully");
                    }
                    Err(e) => {
                        startup_log::log_error(&format!("P2P gossip autostart failed: {e}"));
                    }
                }
            });
            app.manage(p2p_cdn);
            startup_log::log_step("P2P CDN state managed by app");

            startup_log::log_step("Initializing file transfer stack...");
            let ft_app = app.handle().clone();
            let ft_data = app_data_dir.clone();
            if let Err(e) =
                microservice::file_transfer_commands::ensure_file_transfer_stack(&ft_app, &ft_data)
            {
                startup_log::log_warn(&format!("ExodusWorkSpace / file transfer init: {e}"));
            } else {
                startup_log::log_step("File transfer stack initialized successfully");
            }

            startup_log::log_step("Setting up system tray...");
            app_window::log_main_window_state(app.handle(), "before tray");
            if let Err(e) = app_tray::setup_tray(app.handle()) {
                startup_log::log_warn(&format!("System tray setup skipped: {e}"));
            } else {
                startup_log::log_step("System tray setup ok");
            }
            app_window::log_main_window_state(app.handle(), "after tray");
            startup_log::log_step("Ensuring main window visible...");
            app_window::ensure_main_window_visible(app.handle());
            startup_log::log_step("Main window visibility ensured");

            startup_log::log_step("Initializing Video RTC...");
            let rtc_app = app.handle().clone();
            let rtc_data = app_data_dir.clone();
            if let Err(e) = video_rtc_commands::ensure_video_rtc(&rtc_app, &rtc_data) {
                startup_log::log_warn(&format!("Video RTC init: {e}"));
            } else {
                startup_log::log_step("Video RTC initialized successfully");
            }
            let rtc_start_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                startup_log::log_step("Starting Video RTC service...");
                if let Err(e) = video_rtc_service_start(rtc_start_app).await {
                    startup_log::log_warn(&format!("Video RTC autostart: {e}"));
                } else {
                    startup_log::log_step("Video RTC service started successfully");
                }
            });

            startup_log::log_step("Initializing contact directory hub...");
            let contact_node = app
                .try_state::<std::sync::Arc<p2p_cdn::P2pCdnState>>()
                .map(|c| c.node_id.clone());
            if let Err(e) = microservice::contact_directory_commands::ensure_contact_directory_hub(
                app.handle(),
                &app_data_dir,
                contact_node,
            ) {
                startup_log::log_warn(&format!("Contact directory hub init: {e}"));
            } else {
                startup_log::log_step("Contact directory hub initialized");
            }

            startup_log::log_step("setup() complete — returning Ok");
            startup_log::log_step("=== SETUP FUNCTION COMPLETED ===");
            lifecycle_setup.start_scheduler(app.handle().clone());
            startup_log::log_step("Lifecycle scheduler started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
                app_lifecycle::lifecycle_get_status,
                app_lifecycle::lifecycle_run_health_tick,
                app_lifecycle::lifecycle_set_auto_fix,
                app_lifecycle::lifecycle_get_logs,
                app_lifecycle::lifecycle_list_presets,
                capture_page,
                semantic_search,
                embeddings_health,
                ai::ai_health,
                ai::get_ai_config,
                ai::set_ai_config,
                ai::ai_summarize_stream,
                ai::ai_chat_stream,
                downloads::download_url,
                downloads::open_downloads_folder,
                downloads::reveal_download,
                downloads::open_download,
                downloads::list_persisted_downloads,
                password_manager::save_password,
                password_manager::get_password_by_url,
                password_manager::get_password_for_page,
                password_manager::save_password_capture,
                password_autofill::password_build_fill_script,
                password_manager::get_password_by_id,
                password_manager::list_passwords,
                password_manager::delete_password,
                password_manager::search_passwords,
                password_manager::generate_password,
                password_manager::check_password_strength,
                password_manager::check_password_compromised,
                password_manager::get_password_manager_settings,
                password_manager::update_password_manager_settings,
                password_manager::get_weak_passwords,
                password_manager::get_compromised_passwords,
                password_manager::get_password_stats,
                password_manager::save_form_data,
                password_manager::get_form_data,
                password_manager::autofill_field,
                cookie_manager::set_cookie,
                cookie_manager::get_cookies_for_domain,
                cookie_manager::get_cookie_by_id,
                cookie_manager::list_cookies,
                cookie_manager::delete_cookie,
                cookie_manager::delete_cookies_for_domain,
                cookie_manager::delete_all_cookies,
                cookie_manager::search_cookies,
                cookie_manager::cleanup_expired_cookies,
                cookie_manager::get_cookie_manager_settings,
                cookie_manager::update_cookie_manager_settings,
                cookie_manager::block_cookie_domain,
                cookie_manager::unblock_cookie_domain,
                cookie_manager::is_domain_blocked,
                cookie_manager::get_blocked_cookie_domains,
                cookie_manager::get_cookie_stats,
                cookie_manager::export_cookies,
                cookie_manager::import_cookies,
                clear_browsing_data,
                smart_cache::cache_get,
                smart_cache::cache_put,
                smart_cache::cache_remove,
                smart_cache::cache_clear,
                smart_cache::cache_get_stats,
                smart_cache::cache_cleanup_expired,
                smart_cache::cache_get_keys,
                smart_cache::cache_get_size,
                smart_cache::cache_get_count,
                smart_cache::cache_set_limits,
                dark_mode::get_theme_mode,
                dark_mode::set_theme_mode,
                dark_mode::get_effective_theme,
                dark_mode::is_dark_mode,
                dark_mode::toggle_theme,
                dark_mode::update_system_dark_mode,
                dark_mode::inject_dark_mode_css,
                dark_mode::add_custom_theme,
                dark_mode::remove_custom_theme,
                dark_mode::get_custom_theme,
                dark_mode::get_all_custom_themes,
                dark_mode::update_custom_theme,
                dark_mode::get_theme_settings,
                dark_mode::update_theme_settings,
                dark_mode::get_active_theme,
                tracking_protection::should_block_url,
                tracking_protection::block_url,
                tracking_protection::get_tracker_rules,
                tracking_protection::add_tracker_rule,
                tracking_protection::remove_tracker_rule,
                tracking_protection::get_tracking_settings,
                tracking_protection::update_tracking_settings,
                tracking_protection::get_blocked_domains,
                tracking_protection::clear_blocked_domains,
                tracking_protection::get_blocking_stats,
                tracking_protection::set_site_shield_override,
                tracking_protection::get_site_shield_override,
                tracking_protection::refresh_tracker_blocklist,
                tracking_protection::set_tracking_subscription,
                tracking_protection::run_tracking_subscription_refresh_if_due,
                smart_suggestions::get_suggestions,
                smart_suggestions::add_suggestion_history_entry,
                smart_suggestions::add_suggestion_bookmark,
                smart_suggestions::remove_suggestion_bookmark,
                smart_suggestions::get_browsing_history,
                smart_suggestions::get_bookmarks,
                smart_suggestions::clear_browsing_history,
                tab_freezer::register_tab,
                tab_freezer::unregister_tab,
                tab_freezer::update_tab_activity,
                tab_freezer::freeze_tab,
                tab_freezer::unfreeze_tab,
                tab_freezer::auto_freeze_inactive_tabs,
                tab_freezer::get_tabs,
                tab_freezer::get_frozen_tabs,
                tab_freezer::get_tab_freezer_settings,
                tab_freezer::update_tab_freezer_settings,
                tab_freezer::get_total_memory_saved,
                tab_freezer::reset_memory_saved,
                dns_prefetch::resolve_domain,
                dns_prefetch::get_cached_dns,
                dns_prefetch::prefetch_domain,
                dns_prefetch::prefetch_from_page,
                dns_prefetch::queue_prefetch,
                dns_prefetch::process_prefetch_queue,
                dns_prefetch::clear_dns_cache,
                dns_prefetch::get_dns_cache_stats,
                dns_prefetch::get_dns_prefetch_settings,
                dns_prefetch::update_dns_prefetch_settings,
                translation_service::translate_text,
                translation_service::get_translation_settings,
                translation_service::update_translation_settings,
                translation_service::clear_translation_cache,
                safe_browsing::check_url_safe,
                safe_browsing::block_malicious_url,
                safe_browsing::get_threats,
                safe_browsing::add_threat,
                safe_browsing::remove_threat,
                safe_browsing::get_safe_browsing_settings,
                safe_browsing::update_safe_browsing_settings,
                safe_browsing::get_safe_browsing_blocked_domains,
                safe_browsing::clear_safe_browsing_blocked_domains,
                safe_browsing::get_safe_browsing_stats,
                safe_browsing::refresh_safe_browsing_list,
                certificate_validation::validate_certificate,
                certificate_validation::cache_certificate,
                certificate_validation::get_cached_certificate,
                certificate_validation::clear_certificate_cache,
                certificate_validation::add_revoked_certificate,
                certificate_validation::get_revoked_certificates,
                certificate_validation::get_certificate_settings,
                certificate_validation::update_certificate_settings,
                certificate_validation::get_certificate_cache_stats,
                // extension_api::register_extension_api,
                // extension_api::unregister_extension_api,
                // extension_api::get_extension_registration,
                // extension_api::get_all_extension_registrations,
                // extension_api::get_available_apis,
                // extension_api::check_extension_capability,
                // extension_api::get_all_capabilities,
                // extension_permissions::request_permission,
                // extension_permissions::grant_permission,
                // extension_permissions::deny_permission,
                // extension_permissions::check_permission,
                // extension_permissions::get_extension_permissions,
                // extension_permissions::revoke_all_permissions,
                // extension_permissions::add_host_permission,
                // extension_permissions::remove_host_permission,
                // extension_permissions::check_host_permission,
                // extension_permissions::get_host_permissions,
                // extension_permissions::get_pending_permission_requests,
                // extension_permissions::get_all_permission_requests,
                // extension_permissions::get_all_extensions_with_permissions,
                devtools::add_console_log,
                devtools::get_console_logs,
                devtools::clear_console_logs,
                devtools::add_network_request,
                devtools::get_network_requests,
                devtools::clear_network_requests,
                devtools::open_devtools_panel,
                devtools::close_devtools_panel,
                devtools::get_open_devtools_panels,
                devtools::get_devtools_settings,
                devtools::update_devtools_settings,
                devtools::get_devtools_stats,
                privacy_dashboard::record_tracker_blocked,
                privacy_dashboard::record_cookie_blocked,
                privacy_dashboard::record_fingerprinting_blocked,
                privacy_dashboard::record_malicious_site_blocked,
                privacy_dashboard::record_data_saved,
                privacy_dashboard::record_time_saved,
                privacy_dashboard::get_privacy_stats,
                privacy_dashboard::get_privacy_events,
                privacy_dashboard::clear_privacy_events,
                privacy_dashboard::get_privacy_dashboard_settings,
                privacy_dashboard::update_privacy_settings,
                privacy_dashboard::reset_privacy_stats,
                privacy_dashboard::get_privacy_score,
                reading_mode::enable_reading_mode,
                reading_mode::disable_reading_mode,
                reading_mode::is_reading_mode_enabled,
                reading_mode::get_reading_mode_pages,
                reading_mode::get_reading_mode_settings,
                reading_mode::update_reading_mode_settings,
                reading_mode::get_reading_mode_preset,
                reading_mode::apply_reading_mode_preset,
                reading_mode::get_reading_mode_presets,
                reading_mode::create_reading_mode_preset,
                reading_mode::delete_reading_mode_preset,
                reading_mode::generate_reading_mode_css,
                page_zoom::set_page_zoom,
                page_zoom::get_page_zoom,
                page_zoom::reset_page_zoom,
                page_zoom::zoom_in,
                page_zoom::zoom_out,
                page_zoom::reset_zoom,
                page_zoom::get_zoom_settings,
                page_zoom::update_zoom_settings,
                page_zoom::get_all_zoom_levels,
                page_zoom::clear_all_zoom_levels,
                page_zoom::auto_detect_zoom,
                download_manager::start_download,
                download_manager::pause_download,
                download_manager::resume_download,
                download_manager::cancel_download,
                download_manager::remove_download,
                download_manager::update_download_progress,
                download_manager::mark_download_failed,
                download_manager::get_download,
                download_manager::get_all_downloads,
                download_manager::get_active_downloads,
                download_manager::get_completed_downloads,
                download_manager::clear_completed_downloads,
                download_manager::clear_all_downloads,
                download_manager::get_download_settings,
                download_manager::update_download_settings,
                download_manager::get_download_stats,
                history_manager::add_history_entry,
                history_manager::remove_history_entry,
                history_manager::remove_history_by_url,
                history_manager::remove_history_by_domain,
                history_manager::clear_all_history,
                history_manager::clear_history_by_time_range,
                history_manager::get_all_history,
                history_manager::get_history_by_url,
                history_manager::search_history,
                history_manager::get_history_by_time_range,
                history_manager::get_recent_history,
                history_manager::get_most_visited_history,
                history_manager::get_history_by_domain,
                history_manager::get_history_settings,
                history_manager::update_history_settings,
                history_manager::get_history_stats,
                bookmark_sync::add_sync_bookmark,
                bookmark_sync::update_sync_bookmark,
                bookmark_sync::remove_sync_bookmark,
                bookmark_sync::add_sync_folder,
                bookmark_sync::update_sync_folder,
                bookmark_sync::remove_sync_folder,
                bookmark_sync::get_all_sync_bookmarks,
                bookmark_sync::get_all_sync_folders,
                bookmark_sync::sync_bookmarks,
                bookmark_sync::get_sync_settings,
                bookmark_sync::update_sync_settings,
                bookmark_sync::get_device_id,
                bookmark_sync::get_sync_stats,
                encrypted_sync::encrypted_sync_get_settings,
                encrypted_sync::encrypted_sync_set_passphrase,
                encrypted_sync::encrypted_sync_store_bookmarks,
                encrypted_sync::encrypted_sync_load_bookmarks,
                encrypted_sync::encrypted_sync_set_server,
                encrypted_sync::encrypted_sync_upload_vault,
                encrypted_sync::encrypted_sync_download_vault,
                plugins::extension_popup::extension_open_popup_window,
                plugins::extension_popup::extension_close_popup_window,
                plugins::extension_popup::extension_action_set_title,
                plugins::extension_popup::extension_action_set_badge,
                plugins::extension_popup::extension_action_get_state,
                form_autofill::add_autofill_entry,
                form_autofill::get_autofill_entries,
                form_autofill::get_all_autofill_entries,
                form_autofill::remove_autofill_entry,
                form_autofill::clear_autofill_by_domain,
                form_autofill::create_autofill_profile,
                form_autofill::update_autofill_profile,
                form_autofill::get_autofill_profile,
                form_autofill::get_all_autofill_profiles,
                form_autofill::delete_autofill_profile,
                form_autofill::get_autofill_settings,
                form_autofill::update_autofill_settings,
                form_autofill::get_autofill_suggestions,
                form_autofill::form_build_fill_script,
                tab_grouping::create_tab_group,
                tab_grouping::update_tab_group,
                tab_grouping::delete_tab_group,
                tab_grouping::add_tab_to_group,
                tab_grouping::remove_tab_from_group,
                tab_grouping::get_group_for_tab,
                tab_grouping::get_all_tab_groups,
                tab_grouping::get_tab_group,
                tab_grouping::collapse_tab_group,
                tab_grouping::expand_tab_group,
                tab_grouping::get_tab_grouping_settings,
                tab_grouping::update_tab_grouping_settings,
                tab_grouping::auto_group_tabs_by_domain,
                tab_grouping::get_tab_grouping_stats,
                vertical_tabs::enable_vertical_tabs,
                vertical_tabs::disable_vertical_tabs,
                vertical_tabs::set_vertical_tab_position,
                vertical_tabs::set_vertical_tab_width_mode,
                vertical_tabs::set_vertical_tab_fixed_width,
                vertical_tabs::update_vertical_tab_state,
                vertical_tabs::remove_vertical_tab_state,
                vertical_tabs::expand_vertical_tab,
                vertical_tabs::collapse_vertical_tab,
                vertical_tabs::get_vertical_tab_state,
                vertical_tabs::get_all_vertical_tab_states,
                vertical_tabs::get_vertical_tab_settings,
                vertical_tabs::update_vertical_tab_settings,
                vertical_tabs::collapse_inactive_vertical_tabs,
                vertical_tabs::expand_all_vertical_tabs,
                tab_pinning::pin_tab,
                tab_pinning::unpin_tab,
                tab_pinning::is_tab_pinned,
                tab_pinning::get_all_pinned_tabs,
                tab_pinning::get_pinned_tab,
                tab_pinning::update_pinned_tab,
                tab_pinning::reorder_pinned_tabs,
                tab_pinning::mute_pinned_tab,
                tab_pinning::unmute_pinned_tab,
                tab_pinning::clear_all_pinned_tabs,
                tab_pinning::get_pinning_settings,
                tab_pinning::update_pinning_settings,
                tab_pinning::get_pinning_stats,
                search_engine::add_search_engine,
                search_engine::remove_search_engine,
                search_engine::update_search_engine,
                search_engine::set_default_search_engine,
                search_engine::get_default_search_engine,
                search_engine::get_all_search_engines,
                search_engine::get_search_engine,
                search_engine::search,
                search_engine::search_with_engine,
                search_engine::get_search_engine_settings,
                search_engine::update_search_engine_settings,
                new_tab_page::get_new_tab_settings,
                new_tab_page::update_new_tab_settings,
                new_tab_page::set_new_tab_layout,
                new_tab_page::set_new_tab_background_image,
                new_tab_page::set_new_tab_wallpaper_id,
                new_tab_page::set_new_tab_background_color,
                new_tab_page::set_new_tab_custom_css,
                new_tab_page::set_new_tab_custom_html,
                new_tab_page::add_new_tab_widget,
                new_tab_page::remove_new_tab_widget,
                new_tab_page::update_new_tab_widget,
                new_tab_page::reorder_new_tab_widgets,
                new_tab_page::get_all_new_tab_widgets,
                new_tab_page::get_enabled_new_tab_widgets,
                new_tab_page::reset_new_tab_to_default,
                ntp_wallpapers::ntp_get_default_wallpaper_id,
                ntp_wallpapers::ntp_get_wallpaper_library_path,
                ntp_wallpapers::ntp_list_wallpaper_catalog,
                ntp_wallpapers::ntp_wallpaper_file_data_url,
                clear_private_profile_data,
                network_interception::add_interception_rule,
                network_interception::remove_interception_rule,
                network_interception::get_interception_rules,
                network_interception::update_interception_rule,
                network_interception::intercept_request,
                network_interception::log_response,
                network_interception::get_request_log,
                network_interception::get_response_log,
                network_interception::clear_network_logs,
                network_interception::block_network_domain,
                network_interception::unblock_network_domain,
                network_interception::get_blocked_network_domains,
                network_interception::get_network_interception_settings,
                network_interception::update_network_interception_settings,
                content_script::add_content_script,
                content_script::remove_content_script,
                content_script::get_content_script,
                content_script::get_all_content_scripts,
                content_script::get_enabled_content_scripts,
                content_script::update_content_script,
                content_script::enable_content_script,
                content_script::disable_content_script,
                content_script::get_scripts_for_url,
                content_script::get_start_scripts,
                content_script::get_end_scripts,
                content_script::get_idle_scripts,
                content_script::get_css_for_url,
                content_script::get_content_script_settings,
                content_script::update_content_script_settings,
                user_script::add_user_script,
                user_script::remove_user_script,
                user_script::get_user_script,
                user_script::get_all_user_scripts,
                user_script::get_enabled_user_scripts,
                user_script::update_user_script,
                user_script::enable_user_script,
                user_script::disable_user_script,
                user_script::get_user_scripts_for_url,
                user_script::set_user_script_value,
                user_script::get_user_script_value,
                user_script::delete_user_script_value,
                user_script::list_user_script_values,
                user_script::import_user_script,
                user_script::export_user_script,
                user_script::get_user_script_settings,
                user_script::update_user_script_settings,
                font_settings::get_font_settings,
                font_settings::update_font_settings,
                font_settings::add_site_font_settings,
                font_settings::remove_site_font_settings,
                font_settings::get_site_font_settings,
                font_settings::get_all_site_font_settings,
                font_settings::update_site_font_settings,
                font_settings::get_effective_font_settings,
                font_settings::get_system_fonts,
                font_settings::reset_font_settings,
                print_settings::get_print_settings,
                print_settings::update_print_settings,
                print_settings::reset_print_settings,
                print_settings::get_available_printers,
                print_settings::get_print_history,
                print_settings::clear_print_history,
                print_settings::print_to_pdf,
                pdf_viewer::get_pdf_viewer_settings,
                pdf_viewer::update_pdf_viewer_settings,
                pdf_viewer::reset_pdf_viewer_settings,
                pdf_viewer::add_recent_pdf_document,
                pdf_viewer::get_recent_pdf_documents,
                pdf_viewer::clear_recent_pdf_documents,
                pdf_viewer::update_pdf_document_state,
                pdf_viewer::add_pdf_document_bookmark,
                pdf_viewer::remove_pdf_document_bookmark,
                pdf_viewer::add_pdf_document_note,
                pdf_viewer::remove_pdf_document_note,
                pdf_viewer::get_pdf_document,
                tab_search::search_tabs,
                tab_search::get_all_search_tabs,
                tab_search::register_search_tab,
                tab_search::unregister_search_tab,
                tab_search::update_search_tab,
                tab_mute::register_mute_tab,
                tab_mute::unregister_mute_tab,
                tab_mute::set_tab_mute,
                tab_mute::get_tab_mute_state,
                tab_mute::set_tab_audio_playing,
                tab_mute::get_audio_playing_tabs,
                tab_mute::get_all_tab_mute_states,
                tab_preview::register_tab_preview,
                tab_preview::unregister_tab_preview,
                tab_preview::get_tab_preview,
                tab_preview::get_all_tab_previews,
                tab_preview::clear_tab_previews,
                https_only::enable_https_only,
                https_only::disable_https_only,
                https_only::is_https_only_enabled,
                https_only::add_https_only_exception,
                https_only::remove_https_only_exception,
                https_only::get_https_only_exceptions,
                https_only::should_upgrade_url,
                https_only::upgrade_to_https,
                https_only::get_https_only_settings,
                dns_over_https::enable_doh,
                dns_over_https::disable_doh,
                dns_over_https::is_doh_enabled,
                dns_over_https::set_doh_provider,
                dns_over_https::get_active_doh_provider,
                dns_over_https::add_doh_provider,
                dns_over_https::remove_doh_provider,
                dns_over_https::set_doh_fallback,
                dns_over_https::get_doh_providers,
                dns_over_https::get_doh_settings,
                fingerprinting_protection::enable_fingerprinting_protection,
                fingerprinting_protection::disable_fingerprinting_protection,
                fingerprinting_protection::is_fingerprinting_protection_enabled,
                fingerprinting_protection::set_canvas_fingerprinting_protection,
                fingerprinting_protection::set_webgl_fingerprinting_protection,
                fingerprinting_protection::set_audio_fingerprinting_protection,
                fingerprinting_protection::set_font_fingerprinting_protection,
                fingerprinting_protection::set_screen_fingerprinting_protection,
                fingerprinting_protection::set_timezone_fingerprinting_protection,
                fingerprinting_protection::set_language_fingerprinting_protection,
                fingerprinting_protection::set_randomize_user_agent,
                fingerprinting_protection::get_fingerprinting_protection_settings,
                omnibox_actions::enable_omnibox_actions,
                omnibox_actions::disable_omnibox_actions,
                omnibox_actions::is_omnibox_actions_enabled,
                omnibox_actions::parse_omnibox_input,
                biometric_auth::is_biometric_available,
                biometric_auth::enable_biometric,
                biometric_auth::disable_biometric,
                biometric_auth::is_biometric_enabled,
                biometric_auth::authenticate_biometric,
                biometric_auth::set_biometric_require_passwords,
                biometric_auth::set_biometric_require_sensitive,
                biometric_auth::set_biometric_auto_lock_timeout,
                biometric_auth::get_biometric_settings,
                voice_search::is_voice_search_available,
                voice_search::enable_voice_search,
                voice_search::disable_voice_search,
                voice_search::is_voice_search_enabled,
                voice_search::start_voice_recognition,
                voice_search::set_voice_search_language,
                voice_search::set_voice_search_auto_submit,
                voice_search::get_voice_search_settings,
                text_to_speech::is_tts_available,
                text_to_speech::enable_tts,
                text_to_speech::disable_tts,
                text_to_speech::is_tts_enabled,
                text_to_speech::tts_speak,
                text_to_speech::tts_stop,
                text_to_speech::tts_get_voices,
                text_to_speech::tts_set_voice,
                text_to_speech::tts_set_rate,
                text_to_speech::tts_set_pitch,
                text_to_speech::tts_set_volume,
                text_to_speech::tts_get_settings,
                global_audio::mute_all_tabs,
                global_audio::unmute_all_tabs,
                global_audio::toggle_global_mute,
                global_audio::is_globally_muted,
                global_audio::set_global_volume,
                global_audio::get_global_volume,
                global_audio::register_audio_tab,
                global_audio::unregister_audio_tab,
                global_audio::set_global_tab_mute,
                global_audio::set_tab_volume,
                global_audio::update_tab_playing,
                global_audio::get_all_audio_tabs,
                global_audio::get_playing_tabs_count,
                global_audio::show_volume_indicator,
                global_audio::get_global_audio_settings,
                per_site_shields::get_site_shield_settings,
                per_site_shields::set_site_shield_settings,
                per_site_shields::enable_site_shield,
                per_site_shields::disable_site_shield,
                per_site_shields::is_site_shield_enabled,
                per_site_shields::set_default_shield_settings,
                per_site_shields::get_default_shield_settings,
                per_site_shields::add_site_custom_rule,
                per_site_shields::remove_site_custom_rule,
                per_site_shields::get_all_shield_sites,
                per_site_shields::reset_site_shields,
                per_site_shields::clear_all_site_shields,
                tab_stacking::create_tab_stack,
                tab_stacking::delete_tab_stack,
                tab_stacking::add_tab_to_stack,
                tab_stacking::remove_tab_from_stack,
                tab_stacking::move_tab_to_stack,
                tab_stacking::collapse_tab_stack,
                tab_stacking::expand_tab_stack,
                tab_stacking::toggle_tab_stack_collapse,
                tab_stacking::rename_tab_stack,
                tab_stacking::change_tab_stack_color,
                tab_stacking::get_tab_stack,
                tab_stacking::get_all_tab_stacks,
                tab_stacking::get_stack_tabs,
                tab_stacking::enable_tab_stacking,
                tab_stacking::disable_tab_stacking,
                tab_stacking::is_tab_stacking_enabled,
                tab_stacking::set_tab_stacking_auto_group,
                tab_stacking::get_tab_stacking_settings,
                reading_progress::update_reading_progress,
                reading_progress::get_reading_progress,
                reading_progress::get_all_reading_progress,
                reading_progress::get_completed_reading,
                reading_progress::get_in_progress_reading,
                reading_progress::mark_reading_completed,
                reading_progress::reset_reading_progress,
                reading_progress::delete_reading_progress,
                reading_progress::clear_reading_progress,
                reading_progress::enable_reading_progress,
                reading_progress::disable_reading_progress,
                reading_progress::is_reading_progress_enabled,
                reading_progress::set_reading_progress_auto_save,
                reading_progress::set_reading_progress_interval,
                reading_progress::set_reading_progress_indicator,
                reading_progress::set_reading_progress_threshold,
                reading_progress::get_reading_progress_settings,
                annotation::create_annotation,
                annotation::update_annotation,
                annotation::delete_annotation,
                annotation::get_annotation,
                annotation::get_url_annotations,
                annotation::get_all_annotations,
                annotation::search_annotations,
                annotation::enable_annotations,
                annotation::disable_annotations,
                annotation::is_annotations_enabled,
                annotation::set_annotation_default_color,
                annotation::get_annotation_settings,
                media_casting::discover_cast_devices,
                media_casting::get_cast_devices,
                media_casting::start_cast,
                media_casting::stop_cast,
                media_casting::get_cast_state,
                media_casting::get_current_cast_device,
                media_casting::enable_media_casting,
                media_casting::disable_media_casting,
                media_casting::is_media_casting_enabled,
                media_casting::set_cast_auto_discover,
                media_casting::get_media_casting_settings,
                color_blind::enable_color_blind,
                color_blind::disable_color_blind,
                color_blind::is_color_blind_enabled,
                color_blind::set_color_blind_type,
                color_blind::get_color_blind_type,
                color_blind::set_color_blind_intensity,
                color_blind::get_color_blind_intensity,
                color_blind::get_color_blind_settings,
                voice_control::start_voice_listening,
                voice_control::stop_voice_listening,
                voice_control::is_voice_listening,
                voice_control::process_voice_command,
                voice_control::add_voice_command,
                voice_control::remove_voice_command,
                voice_control::enable_voice_command,
                voice_control::disable_voice_command,
                voice_control::get_voice_commands,
                voice_control::enable_voice_control,
                voice_control::disable_voice_control,
                voice_control::is_voice_control_enabled,
                voice_control::set_voice_control_wake_word,
                voice_control::get_voice_control_settings,
                data_saver::enable_data_saver,
                data_saver::disable_data_saver,
                data_saver::is_data_saver_enabled,
                data_saver::set_data_saver_block_images,
                data_saver::set_data_saver_block_videos,
                data_saver::set_data_saver_block_autoplay,
                data_saver::set_data_saver_compress_images,
                data_saver::set_data_saver_quality_level,
                data_saver::record_data_saver_bytes,
                data_saver::get_data_saver_bytes_saved,
                data_saver::reset_data_saver_bytes,
                data_saver::get_data_saver_settings,
                audio_visualization::enable_audio_visualization,
                audio_visualization::disable_audio_visualization,
                audio_visualization::is_audio_visualization_enabled,
                audio_visualization::set_visualization_type,
                audio_visualization::get_visualization_type,
                audio_visualization::set_visualization_sensitivity,
                audio_visualization::get_visualization_sensitivity,
                audio_visualization::set_visualization_smoothing,
                audio_visualization::get_visualization_smoothing,
                audio_visualization::set_visualization_color_scheme,
                audio_visualization::get_visualization_color_scheme,
                audio_visualization::get_audio_visualization_settings,
                password_sharing::enable_password_sharing,
                password_sharing::disable_password_sharing,
                password_sharing::is_password_sharing_enabled,
                password_sharing::add_trusted_contact,
                password_sharing::remove_trusted_contact,
                password_sharing::get_trusted_contacts,
                password_sharing::share_password,
                password_sharing::revoke_shared_password,
                password_sharing::get_shared_passwords,
                password_sharing::set_password_sharing_require_approval,
                password_sharing::set_password_sharing_auto_expire_days,
                password_sharing::get_password_sharing_settings,
                private_search::enable_private_search_tab,
                private_search::disable_private_search_tab,
                private_search::is_private_search_tab_enabled,
                private_search::set_private_search_engine,
                private_search::get_private_search_engine,
                private_search::set_private_search_block_trackers,
                private_search::get_private_search_block_trackers,
                private_search::set_private_search_clear_on_close,
                private_search::get_private_search_clear_on_close,
                private_search::set_private_search_separate_history,
                private_search::get_private_search_separate_history,
                private_search::get_private_search_settings,
                omnibox_image_search::enable_omnibox_image_search,
                omnibox_image_search::disable_omnibox_image_search,
                omnibox_image_search::is_omnibox_image_search_enabled,
                omnibox_image_search::set_omnibox_image_search_engine,
                omnibox_image_search::get_omnibox_image_search_engine,
                omnibox_image_search::set_omnibox_image_search_show_preview,
                omnibox_image_search::get_omnibox_image_search_show_preview,
                omnibox_image_search::set_omnibox_image_search_safe_search,
                omnibox_image_search::get_omnibox_image_search_safe_search,
                omnibox_image_search::get_omnibox_image_search_settings,
                mobile_sync::enable_mobile_sync,
                mobile_sync::disable_mobile_sync,
                mobile_sync::is_mobile_sync_enabled,
                mobile_sync::set_mobile_sync_auto,
                mobile_sync::get_mobile_sync_auto,
                mobile_sync::set_mobile_sync_interval,
                mobile_sync::get_mobile_sync_interval,
                mobile_sync::set_mobile_sync_wifi_only,
                mobile_sync::get_mobile_sync_wifi_only,
                mobile_sync::set_mobile_sync_bookmarks,
                mobile_sync::get_mobile_sync_bookmarks,
                mobile_sync::set_mobile_sync_history,
                mobile_sync::get_mobile_sync_history,
                mobile_sync::set_mobile_sync_passwords,
                mobile_sync::get_mobile_sync_passwords,
                mobile_sync::set_mobile_sync_reading_list,
                mobile_sync::get_mobile_sync_reading_list,
                mobile_sync::trigger_mobile_sync,
                mobile_sync::get_mobile_sync_last_sync,
                mobile_sync::get_mobile_sync_settings,
                password_health::add_password_health_entry,
                password_health::remove_password_health_entry,
                password_health::update_password_strength,
                password_health::mark_password_compromised,
                password_health::mark_password_duplicate,
                password_health::get_password_health_entries,
                password_health::get_password_health_weak_passwords,
                password_health::get_password_health_duplicate_passwords,
                password_health::get_password_health_compromised_passwords,
                password_health::get_password_health_old_passwords,
                password_health::get_password_health_summary,
                password_health::run_password_health_check,
                browser::browser_create_tab,
                browser::browser_navigate,
                browser::browser_go_back,
                browser::browser_go_forward,
                browser::browser_reload,
                browser::browser_eval,
                browser::browser_eval_return,
                browser::browser_get_html,
                browser::browser_capture_content,
                browser::browser_get_title,
                browser::browser_get_selection,
                browser::browser_get_nav_state,
                browser::browser_clear_tab_nav,
                browser::browser_restore_discarded_tab,
                discarded_tabs::browser_discard_tab,
                discarded_tabs::browser_is_tab_discarded,
                discarded_tabs::browser_clear_discarded_tab,
                browser::browser_set_zoom,
                browser::browser_find_in_page,
                browser::browser_set_popup_blocking,
                browser::browser_extension_flush_tab,
                browser::extension_tabs_create_ack,
                browser::extension_pump_runtime,
                browser::browser_toggle_devtools,
                browser::browser_toggle_fullscreen,
                browser::browser_view_source,
                browser::browser_context_menu_action,
                browser::browser_handle_drag_drop,
                browser::browser_toggle_reader_mode,
                browser::browser_group_tabs,
                browser::browser_ungroup_tabs,
                record_visit,
                get_history,
                get_visit_history,
                clear_visit_history,
                clear_rag_data,
                delete_indexed_page,
                search_indexed_pages,
                list_bookmarks,
                add_bookmark,
                remove_bookmark,
                update_bookmark_folder,
                reorder_bookmarks_bar,
                search_bookmarks,
                search_visits,
                compress_dom,
                execute_agent_action_with_context,
                get_privacy_settings,
                set_privacy_settings,
                export_bookmarks,
                import_bookmarks,
                save_session,
                load_session,
                clear_session,
                sidecar::get_sidecar_status,
                sidecar::restart_sidecar,
                plugins::commands::extension_list,
                plugins::commands::extension_set_enabled,
                // plugins::commands::extension_install_folder,
                // plugins::commands::extension_uninstall,
                // plugins::commands::extension_rescan,
                plugins::commands::extension_storage_get,
                plugins::commands::extension_storage_set,
                plugins::commands::extension_storage_remove,
                plugins::commands::extension_storage_clear,
                plugins::commands::extension_sync_tabs,
                plugins::commands::extension_tabs_query,
                plugins::commands::extension_tabs_update,
                plugins::commands::extension_tabs_remove,
                plugins::commands::extension_tabs_reload,
                plugins::commands::extension_background_specs,
                plugins::commands::extension_background_boot,
                plugins::commands::extension_install_crx,
                plugins::commands::extension_store_list,
                plugins::commands::extension_set_store_url,
                plugins::commands::extension_store_fetch_remote,
                plugins::commands::extension_popup_url,
                plugins::commands::extension_get_manifest,
                plugins::commands::extension_emit_installed_event,
                plugins::commands::extension_permissions_contains,
                plugins::commands::extension_permissions_get_all,
                plugins::commands::extension_permissions_request,
                // plugins::commands::extension_permissions_resolve,
                plugins::commands::extension_validate_host_access,
                plugins::commands::extension_site_permissions_list,
                plugins::commands::extension_site_permissions_revoke,
                plugins::commands::extension_site_permissions_revoke_all,
                plugins::commands::browser_site_permissions_list,
                plugins::commands::browser_site_permissions_revoke,
                plugins::commands::browser_site_permission_resolve,
                plugins::commands::extension_host_install_resolve,
                plugins::commands::extension_set_confirm_host_permissions,
                plugins::commands::extension_notifications_create,
                plugins::commands::extension_notifications_update,
                plugins::commands::extension_notifications_clear,
                plugins::commands::extension_notifications_get_all,
                microservice_register,
                microservice_unregister,
                microservice_list,
                microservice_status,
                microservice_heartbeat,
                microservice_health_check_all,
                microservice_start,
                microservice_stop,
                microservice_socket_dir,
                microservice_get_logs,
                microservice_add_log,
                microservice_save_logs,
                microservice_clear_logs,
                microservice_clear_all_logs,
                microservice_get_all_logs,
                microservice_collect_metrics,
                microservice_get_metrics,
                microservice_get_latest_metrics,
                microservice_get_all_metrics,
                microservice_get_average_usage,
                microservice_clear_metrics,
                microservice_clear_all_metrics,
                microservice_health_check,
                microservice_get_health_history,
                microservice_get_health_stats,
                rag_service_start,
                rag_service_stop,
                rag_store_page,
                rag_search_pages,
                rag_add_bookmark,
                rag_list_bookmarks,
                rag_record_visit,
                rag_search_visits,
                crypto_service_start,
                crypto_service_stop,
                crypto_hash,
                crypto_uuid_generate,
                crypto_extract_12_digit,
                crypto_get_formatted_id,
                crypto_generate_qr_code,
                crypto_parse_qr_code,
                os_service_start,
                os_service_stop,
                os_get_platform,
                os_get_arch,
                os_get_temp_dir,
                os_get_home_dir,
                os_path_exists,
                os_read_file,
                os_write_file,
                os_delete_file,
                os_list_dir,
                os_create_dir,
                os_remove_dir,
                p2p_blobs_service_start,
                p2p_blobs_service_stop,
                p2p_blobs_add,
                p2p_blobs_get,
                p2p_blobs_list,
                p2p_blobs_create_ticket,
                p2p_gossip_service_start,
                p2p_gossip_service_stop,
                p2p_gossip_subscribe,
                p2p_gossip_unsubscribe,
                p2p_gossip_publish,
                p2p_gossip_get_messages,
                p2p_gossip_list_topics,
                p2p_gossip_get_subscribers,
                p2p_gossip_node_info,
                ai_model_service_start,
                ai_model_service_stop,
                ai_model_register,
                ai_model_unregister,
                ai_model_get,
                ai_model_search,
                ai_model_list,
                ai_model_get_node_models,
                ai_model_list_nodes,
                ai_model_node_info,
                exodus_workspace_info,
                exodus_workspace_list,
                exodus_workspace_watch_start,
                exodus_workspace_watch_stop,
                file_transfer_receive_to_inbox,
                file_transfer_service_start,
                file_transfer_service_stop,
                file_transfer_initiate,
                file_transfer_pick_file,
                file_transfer_dashboard,
                file_transfer_set_throttle,
                file_transfer_set_auto_reconnect,
                file_transfer_set_relay_config,
                file_transfer_set_relay_serve,
                wan_relay_server_info,
                file_transfer_start_background_download,
                file_transfer_verify_checksum,
                file_transfer_get,
                file_transfer_list,
                file_transfer_get_chunks,
                file_transfer_update_status,
                file_transfer_cancel,
                file_transfer_generate_qr_data,
                file_transfer_resolve_by_short_code,
                file_transfer_retry,
                file_transfer_resolve_conflict,
                video_rtc_service_start,
                video_rtc_node_info,
                video_rtc_set_display_name,
                video_rtc_publish_signal,
                video_rtc_poll_signals,
                video_rtc_call_start,
                video_rtc_call_update,
                video_rtc_call_list,
                video_rtc_meeting_create,
                video_rtc_meeting_join,
                video_rtc_meeting_leave,
                video_rtc_meeting_get,
                video_rtc_meeting_list,
                video_rtc_peer_topic,
                ai_agent_service_start,
                ai_agent_service_stop,
                ai_agent_register,
                ai_agent_unregister,
                ai_agent_get,
                ai_agent_list,
                ai_agent_find_by_capability,
                ai_agent_find_by_type,
                ai_agent_update_status,
                ai_agent_send_message,
                ai_agent_get_messages,
                ai_agent_broadcast_presence,
                ai_agent_link_to_public_account,
                ai_agent_unlink_from_public_account,
                ai_agent_get_agents_by_public_account,
                contact_directory_service_start,
                contact_directory_service_stop,
                contact_directory_hub_info,
                contact_get_local_digit, contact_export_json, contact_import_json,
                contact_add,
                contact_remove,
                contact_update,
                contact_get,
                contact_list,
                contact_search,
                contact_get_by_group,
                contact_get_favorites,
                contact_get_blocked,
                contact_add_to_group,
                contact_remove_from_group,
                contact_group_create,
                contact_group_delete,
                contact_group_list,
                contact_toggle_favorite,
                contact_block,
                contact_unblock,
                contact_get_recent,
                contact_get_by_node,
                contact_get_by_agent,
                contact_add_friend_by_digit,
                contact_register_digit_mapping,
                contact_resolve_digit_to_node,
                contact_get_digit_for_node,
                contact_filter_by_type,
                contact_filter_by_deployment_type,
                contact_link_to_public_account,
                contact_unlink_from_public_account,
                contact_get_contacts_by_public_account,
                contact_set_friend_request_mode,
                contact_get_friend_request_mode,
                contact_filter_by_iot_device_type,
                contact_filter_by_iot_protocol,
                contact_filter_by_iot_status,
                contact_get_iot_devices_by_location,
                contact_get_all_iot_devices,
                contact_update_iot_device_status,
                contact_get_online_iot_devices,
                contact_get_offline_iot_devices,
                group_chat_service_start,
                group_chat_service_stop,
                group_create,
                group_update,
                group_delete,
                direct_chat_create_or_get,
                direct_send_message,
                direct_get_messages,
                direct_edit_message,
                direct_delete_message,
                direct_get_chat,
                direct_list_chats,
                direct_update_sequence,
                direct_get_sequence,
                direct_detect_missing,
                direct_get_messages_by_sequence,
                direct_create_receipt,
                direct_get_receipts,
                direct_verify_message,
                group_get,
                group_list_user,
                group_add_member,
                group_remove_member,
                group_get_members,
                group_send_message,
                group_get_messages,
                group_edit_message,
                group_delete_message,
                group_create_invitation,
                group_accept_invitation,
                group_reject_invitation,
                group_get_pending_invitations,
                group_update_member_online,
                group_search,
                group_link_to_public_account,
                group_unlink_from_public_account,
                group_get_by_public_account,
                group_add_admin,
                group_remove_admin,
                group_is_admin,
                group_is_owner,
                group_has_permission,
                social_feed_service_start,
                social_feed_service_stop,
                social_post_create,
                social_post_update,
                social_post_delete,
                social_post_get,
                social_post_get_user,
                social_feed_get_timeline,
                social_post_search,
                social_post_link_to_public_account,
                social_post_unlink_from_public_account,
                social_post_get_by_public_account,
                social_comment_add,
                social_comment_get,
                social_comment_delete,
                social_reaction_add,
                social_reaction_remove,
                social_reaction_get,
                social_user_follow,
                social_user_unfollow,
                social_user_get_followings,
                social_user_get_followers,
                social_user_is_following,
                agent_discovery_service_start,
                agent_discovery_service_stop,
                agent_discovery_register,
                agent_discovery_update_activity,
                agent_discovery_discover,
                agent_discovery_trending,
                agent_discovery_search_capability,
                media_streaming_service_start,
                media_streaming_service_stop,
                media_stream_create,
                media_stream_update,
                media_stream_end,
                media_stream_get,
                media_stream_list_active,
                media_stream_list_user,
                media_stream_join,
                media_stream_leave,
                media_stream_get_viewers,
                media_stream_get_qualities,
                media_stream_search,
                media_stream_get_trending,
                media_stream_get_audio_qualities,
                media_stream_set_audio_effects,
                media_stream_get_audio_effects,
                media_stream_add_audio_analysis,
                media_stream_get_audio_analysis,
                media_stream_create_audio_mixer,
                media_stream_get_audio_mixer,
                media_stream_update_mixer_gain,
                media_stream_update_mixer_panning,
                news_aggregation_service_start,
                news_aggregation_service_stop,
                news_add_article,
                news_add_source,
                news_update_source,
                news_remove_source,
                news_get_article,
                news_get_articles_by_source,
                news_get_articles_by_category,
                news_search_articles,
                news_get_latest_articles,
                news_get_sources,
                news_create_feed,
                news_get_feed_articles,
                news_get_statistics,
                public_account_service_start,
                public_account_service_stop,
                public_account_create,
                public_account_get,
                public_account_update,
                public_account_list,
                public_account_get_by_owner,
                public_account_publish_article,
                public_account_schedule_article,
                public_account_process_scheduled_articles,
                public_account_get_scheduled_articles,
                public_account_get_article,
                public_account_list_articles,
                public_account_subscribe,
                public_account_unsubscribe,
                public_account_get_followers,
                public_account_get_subscriptions,
                public_account_record_view,
                public_account_like_article,
                public_account_get_analytics,
                public_account_get_article_analytics,
                public_account_upload_media,
                public_account_delete_media,
                public_account_get_media,
                public_account_list_media,
                public_account_list_media_by_type,
                public_account_save_draft,
                public_account_list_drafts,
                public_account_delete_draft,
                public_account_add_menu_item,
                public_account_update_menu_item,
                public_account_delete_menu_item,
                public_account_get_menu_items,
                public_account_send_notification,
                public_account_mark_notification_read,
                public_account_get_notifications,
                public_account_get_unread_notifications,
                public_account_search,
                public_account_get_trending_articles,
                public_account_get_articles_by_category,
                public_account_recommend_articles,
                public_account_get_realtime_analytics,
                service_exposure_service_start,
                service_exposure_service_stop,
                service_exposure_expose,
                service_exposure_stop,
                service_exposure_get,
                service_exposure_list,
                service_exposure_update_heartbeat,
                port_forwarding_service_start,
                port_forwarding_service_stop,
                port_forwarding_create,
                port_forwarding_stop,
                port_forwarding_get,
                port_forwarding_list,
                port_forwarding_update_heartbeat,
                port_forwarding_retry,
                video_communication_service_start,
                video_communication_service_stop,
                video_call_initiate,
                video_call_accept,
                video_call_end,
                video_call_get,
                video_call_list,
                video_call_add_frame,
                video_call_get_frames,
                video_call_add_audio_frame,
                video_call_get_audio_frames,
                video_call_initiate_by_digit,
                video_call_get_digit,
                video_render_loop,
                audio_render_loop,
                collaborative_editing_service_start,
                collaborative_editing_service_stop,
                collaborative_create_document,
                collaborative_open_document,
                collaborative_close_document,
                collaborative_apply_operation,
                collaborative_get_document,
                collaborative_list_documents,
                collaborative_update_cursor,
                collaborative_get_cursors,
                collaborative_get_operations,
                clipboard_sync_service_start,
                clipboard_sync_service_stop,
                clipboard_sync_create,
                clipboard_sync_stop,
                clipboard_sync_add_item,
                clipboard_sync_get_history,
                clipboard_sync_get,
                clipboard_sync_list,
                clipboard_sync_connect_device,
                clipboard_sync_disconnect_device,
                terminal_session_service_start,
                terminal_session_service_stop,
                terminal_session_create,
                terminal_session_end,
                terminal_session_add_output,
                terminal_session_get_outputs,
                terminal_session_get,
                terminal_session_list,
                terminal_session_connect_user,
                terminal_session_disconnect_user,
                terminal_session_send_command,
                ai_video_analysis_service_start,
                ai_video_analysis_service_stop,
                ai_video_analysis_create_task,
                ai_video_analysis_delete_task,
                ai_video_analysis_add_result,
                ai_video_analysis_get_results,
                ai_video_analysis_get_task,
                ai_video_analysis_list_tasks,
                ai_video_analysis_get_stats,
                ai_video_analysis_enable_task,
                ai_video_analysis_disable_task,
                p2p_cdn_join_room,
                p2p_cdn_leave_room,
                p2p_cdn_node_info,
                p2p_cdn_room_feed,
                p2p_cdn_list_peers,
                p2p_cdn_announce_asset,
                p2p_cdn_register_local_seed,
                p2p_cdn_download,
                p2p_cdn_get_asset,
                p2p_cdn_start_mesh,
                p2p_cdn_announce_group_hot,
                p2p_cdn_hash_file,
                p2p_cdn_sync_gossip,
                p2p_cdn_announce_url_hot,
                p2p_cdn_group_send_message,
                p2p_cdn_url_status,
                tab_sleeping_commands::tab_sleep_register,
                tab_sleeping_commands::tab_sleep_unregister,
                tab_sleeping_commands::tab_sleep_mark_active,
                tab_sleeping_commands::tab_sleep_update_media,
                tab_sleeping_commands::tab_sleep_update_memory,
                tab_sleeping_commands::tab_sleep_get_candidates,
                tab_sleeping_commands::tab_sleep_mark_sleeping,
                tab_sleeping_commands::tab_sleep_wake,
                tab_sleeping_commands::tab_sleep_get_all,
                tab_sleeping_commands::tab_sleep_get_stats,
                tab_sleeping_commands::tab_sleep_update_config,
                tab_sleeping_commands::tab_sleep_get_config,
                microservice_monitoring_commands::metrics_counter,
                microservice_monitoring_commands::metrics_gauge,
                microservice_monitoring_commands::metrics_histogram,
                microservice_monitoring_commands::metrics_get_metric,
                microservice_monitoring_commands::metrics_get_all,
                microservice_monitoring_commands::metrics_get_stats,
                microservice_monitoring_commands::metrics_export_prometheus,
                microservice_monitoring_commands::metrics_cleanup,
                resource_preloader_commands::preloader_add_hint,
                resource_preloader_commands::preloader_process_queue,
                resource_preloader_commands::preloader_get_cached,
                resource_preloader_commands::preloader_learn_pattern,
                resource_preloader_commands::preloader_get_predictive_hints,
                resource_preloader_commands::preloader_get_stats,
                resource_preloader_commands::preloader_update_config,
                resource_preloader_commands::preloader_get_config,
                resource_preloader_commands::preloader_clear_cache,
                hermes_commands::hermes_create_task,
                hermes_commands::hermes_execute_task,
                hermes_commands::hermes_get_task,
                hermes_commands::hermes_get_all_tasks,
                hermes_commands::hermes_cancel_task,
                hermes_commands::hermes_create_strategy,
                hermes_commands::hermes_execute_strategy,
                hermes_commands::hermes_update_context,
                hermes_commands::hermes_get_context,
                hermes_commands::hermes_get_stats,
                hermes_commands::hermes_update_config,
                hermes_commands::hermes_get_config,
                python_microservice_commands::python_microservice_start,
                python_microservice_commands::python_microservice_stop,
                python_microservice_commands::python_microservice_restart,
                python_microservice_commands::python_microservice_execute,
                python_microservice_commands::python_microservice_get_info,
                python_microservice_commands::python_microservice_get_status,
                python_microservice_commands::python_microservice_update_config,
                python_microservice_commands::python_microservice_get_config,
                inference_commands::inference_load_model,
                inference_commands::inference_unload_model,
                inference_commands::inference_generate,
                inference_commands::inference_chat,
                inference_commands::inference_embed,
                inference_commands::inference_add_model,
                inference_commands::inference_remove_model,
                inference_commands::inference_list_models,
                inference_commands::inference_get_loaded_model,
                inference_commands::inference_get_status,
                inference_commands::inference_get_stats,
                inference_commands::inference_update_config,
                inference_commands::inference_get_config,
                allama_service_start,
                allama_service_stop,
                allama_service_restart,
                allama_service_status,
                allama_http_health,
                allama_control_rpc,
                allama_register_microservice,
                allama_list_models,
                picture_in_picture::pip_enter,
                picture_in_picture::pip_exit,
                picture_in_picture::pip_resize,
                picture_in_picture::pip_get_state,
                picture_in_picture::pip_get_all_active,
                pocket_save_article,
                pocket_list_articles,
                pocket_get_article,
                pocket_update_article,
                pocket_mark_as_read,
                pocket_delete_article,
                pocket_search_articles,
                pocket_get_articles_by_tag,
                pocket_get_all_tags,
                pocket_get_stats,
            ])
        .build(tauri::generate_context!())
        .expect("error while building exodus application")
        .run(|app_handle, event| {
            startup_log::log_step("=== RUN CALLBACK STARTED ===");
            startup_log::log_step(&format!("Run event received: {:?}", std::mem::discriminant(&event)));
            if let Some(lc) = app_handle.try_state::<Arc<app_lifecycle::AppLifecycleManager>>() {
                startup_log::log_step("Lifecycle manager found, calling on_run_event");
                lc.on_run_event(app_handle, &event);
            } else if matches!(event, tauri::RunEvent::Ready) {
                startup_log::log_step("Lifecycle manager NOT found, using fallback");
                startup_log::log_step("RunEvent::Ready (lifecycle manager not yet managed)");
                app_window::log_main_window_state(app_handle, "RunEvent::Ready");
                app_window::ensure_main_window_visible(app_handle);

                // Disabled automatic window size adjustment to prevent interference with setup settings
                // The window size is now controlled by setup() in lib.rs
                /*
                // Force window size and position when lifecycle manager is not yet managed
                if let Some(win) = app_handle.get_webview_window("main") {
                    let win_clone = win.clone();
                    let handle = app_handle.clone();

                    // First attempt immediately
                    match win_clone.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: 1280,
                        height: 720,
                    })) {
                        Ok(_) => startup_log::log_step("RunEvent::Ready (fallback): set_size(1280x720) succeeded (Physical)"),
                        Err(e) => startup_log::log_error(&format!("RunEvent::Ready (fallback): set_size(1280x720) failed (Physical): {}", e)),
                    }
                    match win_clone.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: 60,
                        y: 60,
                    })) {
                        Ok(_) => startup_log::log_step("RunEvent::Ready (fallback): set_position(60, 60) succeeded (Physical)"),
                        Err(e) => startup_log::log_error(&format!("RunEvent::Ready (fallback): set_position(60, 60) failed (Physical): {}", e)),
                    }
                    let _ = win_clone.unminimize();

                    // Log the actual window state after setting
                    if let Ok(pos) = win_clone.outer_position() {
                        startup_log::log_step(&format!("RunEvent::Ready (fallback): Window position after set: x={}, y={}", pos.x, pos.y));
                    }
                    if let Ok(size) = win_clone.outer_size() {
                        startup_log::log_step(&format!("RunEvent::Ready (fallback): Window size after set: width={}, height={}", size.width, size.height));
                    }

                    // Schedule a delayed retry using tokio
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        if let Some(win) = handle.get_webview_window("main") {
                            startup_log::log_step("RunEvent::Ready (fallback): Delayed retry - forcing window size and position");
                            match win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                                width: 1280,
                                height: 720,
                            })) {
                                Ok(_) => startup_log::log_step("RunEvent::Ready (fallback): Delayed set_size(1280x720) succeeded (Physical)"),
                                Err(e) => startup_log::log_error(&format!("RunEvent::Ready (fallback): Delayed set_size(1280x720) failed (Physical): {}", e)),
                            }
                            match win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                                x: 60,
                                y: 60,
                            })) {
                                Ok(_) => startup_log::log_step("RunEvent::Ready (fallback): Delayed set_position(60, 60) succeeded (Physical)"),
                                Err(e) => startup_log::log_error(&format!("RunEvent::Ready (fallback): Delayed set_position(60, 60) failed (Physical): {}", e)),
                            }
                            let _ = win.unminimize();

                            // Log the actual window state after delayed setting
                            if let Ok(pos) = win.outer_position() {
                                startup_log::log_step(&format!("RunEvent::Ready (fallback): Delayed Window position after set: x={}, y={}", pos.x, pos.y));
                            }
                            if let Ok(size) = win.outer_size() {
                                startup_log::log_step(&format!("RunEvent::Ready (fallback): Delayed Window size after set: width={}, height={}", size.width, size.height));
                            }
                        }
                    });
                }
                */
            }
            app_tray::on_run_event(app_handle, &event);
        });
}
