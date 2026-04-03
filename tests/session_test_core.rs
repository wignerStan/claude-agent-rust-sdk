//! Tests for session management: Session and SessionManager.

use claude_agent::core::session::{Session, SessionManager};

#[test]
fn session_new_generates_unique_uuids() {
    let s1 = Session::new();
    let s2 = Session::new();
    assert_ne!(s1.id, s2.id);
    assert_eq!(s1.id.len(), 36);
}

#[test]
fn session_new_defaults() {
    let session = Session::new();
    assert!(session.is_active);
    assert!(session.checkpoints.is_empty());
    assert!(session.metadata.is_empty());
}

#[test]
fn session_default_trait() {
    let s = Session::default();
    assert!(s.is_active);
}

#[test]
fn session_with_id_str() {
    let session = Session::with_id("custom-42");
    assert_eq!(session.id, "custom-42");
    assert!(session.is_active);
}

#[test]
fn session_with_id_string() {
    let session = Session::with_id(String::from("owned"));
    assert_eq!(session.id, "owned");
}

#[test]
fn session_add_checkpoint() {
    let mut session = Session::new();
    assert!(session.last_checkpoint().is_none());
    session.add_checkpoint("msg-1".to_string());
    let cp = session.last_checkpoint().unwrap();
    assert_eq!(cp.user_message_id, "msg-1");
    assert!(cp.timestamp > 0);
}

#[test]
fn session_multiple_checkpoints_ordered() {
    let mut session = Session::new();
    session.add_checkpoint("a".to_string());
    session.add_checkpoint("b".to_string());
    session.add_checkpoint("c".to_string());
    assert_eq!(session.checkpoints.len(), 3);
    assert_eq!(session.last_checkpoint().unwrap().user_message_id, "c");
}

#[test]
fn session_deactivate() {
    let mut session = Session::new();
    session.deactivate();
    assert!(!session.is_active);
}

#[test]
fn session_fork_new_id() {
    let session = Session::with_id("orig");
    let forked = session.fork();
    assert_ne!(forked.id, session.id);
    assert!(forked.is_active);
}

#[test]
fn session_fork_copies_checkpoints() {
    let mut session = Session::new();
    session.add_checkpoint("cp1".to_string());
    session.add_checkpoint("cp2".to_string());
    let forked = session.fork();
    assert_eq!(forked.checkpoints.len(), 2);
    assert_eq!(forked.checkpoints[0].user_message_id, "cp1");
}

#[test]
fn session_fork_copies_metadata() {
    let mut session = Session::new();
    session.metadata.insert("k".to_string(), serde_json::json!("v"));
    let forked = session.fork();
    assert_eq!(forked.metadata.get("k"), Some(&serde_json::json!("v")));
}

#[test]
fn session_fork_independent() {
    let mut session = Session::new();
    session.add_checkpoint("x".to_string());
    let mut forked = session.fork();
    forked.add_checkpoint("y".to_string());
    forked.deactivate();
    assert_eq!(session.checkpoints.len(), 1);
    assert!(session.is_active);
    assert_eq!(forked.checkpoints.len(), 2);
    assert!(!forked.is_active);
}

// SessionManager tests

#[test]
fn manager_new() {
    let mgr = SessionManager::new();
    assert!(mgr.current_session().is_none());
}

#[test]
fn manager_default() {
    assert!(SessionManager::default().current_session().is_none());
}

#[test]
fn manager_create_session() {
    let mut mgr = SessionManager::new();
    let session = mgr.create_session();
    assert!(session.is_active);
    assert!(!session.id.is_empty());
    assert!(mgr.current_session().is_some());
}

#[test]
fn manager_create_multiple_current_is_last() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create_session().id.clone();
    let id2 = mgr.create_session().id.clone();
    assert_ne!(id1, id2);
    assert_eq!(mgr.current_session().unwrap().id, id2);
}

#[test]
fn manager_get_session_found() {
    let mut mgr = SessionManager::new();
    let id = mgr.create_session().id.clone();
    assert!(mgr.get_session(&id).is_some());
    assert_eq!(mgr.get_session(&id).unwrap().id, id);
}

#[test]
fn manager_get_session_not_found() {
    let mgr = SessionManager::new();
    assert!(mgr.get_session("nope").is_none());
}

#[test]
fn manager_resume_session() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create_session().id.clone();
    let _id2 = mgr.create_session();
    let resumed = mgr.resume_session(&id1);
    assert!(resumed.is_some());
    assert_eq!(mgr.current_session().unwrap().id, id1);
}

#[test]
fn manager_resume_nonexistent() {
    let mut mgr = SessionManager::new();
    mgr.create_session();
    assert!(mgr.resume_session("ghost").is_none());
}

#[test]
fn manager_current_session_mut() {
    let mut mgr = SessionManager::new();
    assert!(mgr.current_session_mut().is_none());
    mgr.create_session();
    mgr.current_session_mut().unwrap().deactivate();
    assert!(!mgr.current_session().unwrap().is_active);
}
