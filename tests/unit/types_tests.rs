//! Tests for ChippMessage, ChippSession, and MessageRole types.

use chipp::{ChippMessage, ChippSession, MessageRole};

// ============================================================================
// MessageRole Tests
// ============================================================================

#[test]
fn test_message_role_user_serializes_lowercase() {
    let role = MessageRole::User;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, r#""user""#);
}

#[test]
fn test_message_role_assistant_serializes_lowercase() {
    let role = MessageRole::Assistant;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, r#""assistant""#);
}

#[test]
fn test_message_role_system_serializes_lowercase() {
    let role = MessageRole::System;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, r#""system""#);
}

#[test]
fn test_message_role_deserializes_from_lowercase() {
    let user: MessageRole = serde_json::from_str(r#""user""#).unwrap();
    let assistant: MessageRole = serde_json::from_str(r#""assistant""#).unwrap();
    let system: MessageRole = serde_json::from_str(r#""system""#).unwrap();

    assert_eq!(user, MessageRole::User);
    assert_eq!(assistant, MessageRole::Assistant);
    assert_eq!(system, MessageRole::System);
}

#[test]
fn test_message_role_equality() {
    assert_eq!(MessageRole::User, MessageRole::User);
    assert_ne!(MessageRole::User, MessageRole::Assistant);
    assert_ne!(MessageRole::Assistant, MessageRole::System);
}

#[test]
fn test_message_role_clone() {
    let role = MessageRole::User;
    let cloned = role.clone();
    assert_eq!(role, cloned);
}

#[test]
fn test_message_role_debug() {
    let debug = format!("{:?}", MessageRole::User);
    assert_eq!(debug, "User");
}

// ============================================================================
// ChippMessage Tests
// ============================================================================

#[test]
fn test_message_user_constructor() {
    let msg = ChippMessage::user("Hello!");
    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.content, "Hello!");
}

#[test]
fn test_message_assistant_constructor() {
    let msg = ChippMessage::assistant("Hi there!");
    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.content, "Hi there!");
}

#[test]
fn test_message_system_constructor() {
    let msg = ChippMessage::system("You are a helpful assistant.");
    assert_eq!(msg.role, MessageRole::System);
    assert_eq!(msg.content, "You are a helpful assistant.");
}

#[test]
fn test_message_constructors_accept_string() {
    let content = String::from("Test message");
    let msg = ChippMessage::user(content);
    assert_eq!(msg.content, "Test message");
}

#[test]
fn test_message_serializes_correctly() {
    let msg = ChippMessage::user("Hello!");
    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains(r#""role":"user""#));
    assert!(json.contains(r#""content":"Hello!""#));
}

#[test]
fn test_message_deserializes_correctly() {
    let json = r#"{"role":"assistant","content":"Response"}"#;
    let msg: ChippMessage = serde_json::from_str(json).unwrap();

    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.content, "Response");
}

#[test]
fn test_message_clone() {
    let msg = ChippMessage::user("Original");
    let cloned = msg.clone();

    assert_eq!(msg.content, cloned.content);
    assert_eq!(msg.role, cloned.role);
}

#[test]
fn test_message_debug() {
    let msg = ChippMessage::user("Test");
    let debug = format!("{:?}", msg);

    assert!(debug.contains("ChippMessage"));
    assert!(debug.contains("User"));
    assert!(debug.contains("Test"));
}

// ============================================================================
// ChippSession Tests
// ============================================================================

#[test]
fn test_session_new_has_no_id() {
    let session = ChippSession::new();
    assert!(session.chat_session_id.is_none());
}

#[test]
fn test_session_with_id_stores_id() {
    let session = ChippSession::with_id("test-session-123");
    assert_eq!(
        session.chat_session_id,
        Some("test-session-123".to_string())
    );
}

#[test]
fn test_session_with_id_accepts_string() {
    let id = String::from("my-session");
    let session = ChippSession::with_id(id);
    assert_eq!(session.chat_session_id, Some("my-session".to_string()));
}

#[test]
fn test_session_reset_clears_id() {
    let mut session = ChippSession::with_id("test-id");
    assert!(session.chat_session_id.is_some());

    session.reset();
    assert!(session.chat_session_id.is_none());
}

#[test]
fn test_session_default_is_empty() {
    let session = ChippSession::default();
    assert!(session.chat_session_id.is_none());
}

#[test]
fn test_session_clone() {
    let session = ChippSession::with_id("clone-me");
    let cloned = session.clone();

    assert_eq!(session.chat_session_id, cloned.chat_session_id);
}

#[test]
fn test_session_debug() {
    let session = ChippSession::with_id("debug-id");
    let debug = format!("{:?}", session);

    assert!(debug.contains("ChippSession"));
    assert!(debug.contains("debug-id"));
}
