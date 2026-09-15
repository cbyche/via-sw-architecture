//! [`ConversationSync`] — ported from `server/test/conversation-sync.test.mjs`,
//! plus retention, the owning task and its shutdown.

use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;
use via_conversation::sync::{
    ConversationClosed, ConversationSync, ConversationSyncHandle, InputReference, MessageRole,
    MessageSource, RecordInput, Retention, SessionRef, equivalent_speech, speech_key,
};

fn contents(messages: &[via_conversation::Message]) -> Vec<String> {
    messages
        .iter()
        .map(|message| message.content.clone())
        .collect()
}

fn user(owner: &str, session: &str, id: &str, content: &str) -> RecordInput {
    RecordInput::new(
        owner,
        session,
        id,
        MessageRole::User,
        content,
        MessageSource::VoiceUser,
    )
}

#[test]
fn recent_context_is_isolated_by_owner_and_by_session() {
    let mut sync = ConversationSync::default();
    sync.record(user("owner-one", "voice-one", "user-one", "继续首页"));
    sync.record(user("owner-two", "voice-one", "user-two", "其他人的内容"));
    sync.record(user("owner-one", "voice-two", "user-three", "另一个会话"));

    assert_eq!(
        contents(&sync.frontend_context(&SessionRef::new("owner-one", "voice-one"))),
        vec!["继续首页".to_owned()]
    );
    assert_eq!(
        contents(&sync.frontend_context(&SessionRef::new("owner-two", "voice-one"))),
        vec!["其他人的内容".to_owned()]
    );
}

#[test]
fn the_same_id_updates_in_place_and_keeps_its_position() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    let first = sync
        .record(user("owner", "voice", "a", "第一条"))
        .expect("recorded");
    sync.record(user("owner", "voice", "b", "第二条"));

    let updated = sync
        .record(RecordInput::new(
            "owner",
            "voice",
            "a",
            MessageRole::Assistant,
            "改过的第一条",
            MessageSource::AgentPresentation,
        ))
        .expect("updated");

    assert_eq!(sync.list(&session).len(), 2);
    assert_eq!(updated.seq, first.seq, "seq is assigned once");
    assert_eq!(updated.created_at, first.created_at, "so is created_at");
    assert_eq!(updated.role, MessageRole::Assistant);
    assert_eq!(
        contents(&sync.list(&session)),
        vec!["改过的第一条".to_owned(), "第二条".to_owned()],
        "an update does not move the message to the end"
    );
}

#[test]
fn a_message_with_no_id_or_no_text_is_not_recorded() {
    let mut sync = ConversationSync::default();
    assert!(sync.record(user("owner", "voice", "", "内容")).is_none());
    assert!(sync.record(user("owner", "voice", "a", "   ")).is_none());
    assert!(sync.record(user("owner", "voice", "a", "\n\t")).is_none());
    assert_eq!(sync.sequence(), 0, "a refused message consumes no seq");
    assert!(sync.list(&SessionRef::new("owner", "voice")).is_empty());
}

#[test]
fn input_references_are_cloned_on_the_way_in_and_out() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    let inputs = vec![InputReference {
        reference: "input_1".to_owned(),
        kind: "image".to_owned(),
        label: "[Image 1]".to_owned(),
        filename: "cat.png".to_owned(),
        mime: "image/png".to_owned(),
    }];
    sync.record(user("owner", "voice", "image-turn", "[Image 1]").inputs(inputs.clone()));

    let mut taken = sync.frontend_context(&session);
    assert_eq!(taken[0].inputs[0].reference, "input_1");
    // Mutating the copy the caller got back cannot reach the store.
    taken[0].inputs[0].reference = "mutated".to_owned();
    assert_eq!(
        sync.frontend_context(&session)[0].inputs[0].reference,
        "input_1"
    );
}

#[test]
fn a_typed_multimodal_turn_is_restored_like_a_spoken_one() {
    let mut sync = ConversationSync::default();
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "typed-image-turn",
            MessageRole::User,
            "[Image 1]",
            MessageSource::TextUser,
        )
        .inputs(vec![InputReference {
            reference: "input_1".to_owned(),
            kind: "image".to_owned(),
            label: "[Image 1]".to_owned(),
            ..InputReference::default()
        }]),
    );

    let restored = sync.frontend_context(&SessionRef::new("owner", "voice"));
    assert_eq!(restored[0].inputs[0].reference, "input_1");
    assert_eq!(restored[0].source, MessageSource::TextUser);
}

#[test]
fn an_agent_result_is_replayed_only_when_no_presentation_covered_it() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "agent:work-presented",
            MessageRole::Assistant,
            "已经说过的结果",
            MessageSource::AgentResult,
        )
        .task("work-presented"),
    );
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "voice:assistant:r1",
            MessageRole::Assistant,
            "我说出来的版本",
            MessageSource::AgentPresentation,
        )
        .task("work-presented"),
    );
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "agent:work-silent",
            MessageRole::Assistant,
            "还没说过的结果",
            MessageSource::AgentResult,
        )
        .task("work-silent"),
    );

    assert_eq!(
        contents(&sync.frontend_context(&session)),
        vec!["我说出来的版本".to_owned(), "还没说过的结果".to_owned()],
        "the model is never told about the same finished Work twice"
    );
}

#[test]
fn a_batched_presentation_covers_every_work_it_names() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    for id in ["w1", "w2"] {
        sync.record(
            RecordInput::new(
                "owner",
                "voice",
                format!("agent:{id}"),
                MessageRole::Assistant,
                format!("结果 {id}"),
                MessageSource::AgentResult,
            )
            .task(id),
        );
    }
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "voice:assistant:batched",
            MessageRole::Assistant,
            "两件都做完了",
            MessageSource::AgentPresentation,
        )
        .tasks(vec!["w1".to_owned(), "w2".to_owned()]),
    );

    assert_eq!(
        contents(&sync.frontend_context(&session)),
        vec!["两件都做完了".to_owned()]
    );
}

#[test]
fn an_agent_result_with_no_work_id_is_never_replayed() {
    let mut sync = ConversationSync::default();
    sync.record(RecordInput::new(
        "owner",
        "voice",
        "orphan",
        MessageRole::Assistant,
        "没有 work_id 的结果",
        MessageSource::AgentResult,
    ));
    assert!(
        sync.frontend_context(&SessionRef::new("owner", "voice"))
            .is_empty()
    );
}

#[test]
fn task_ids_are_deduplicated_and_emptied_of_blanks() {
    let mut sync = ConversationSync::default();
    let message = sync
        .record(
            RecordInput::new(
                "owner",
                "voice",
                "a",
                MessageRole::Assistant,
                "内容",
                MessageSource::AgentPresentation,
            )
            .tasks(vec![
                "w1".to_owned(),
                String::new(),
                "w1".to_owned(),
                "w2".to_owned(),
            ]),
        )
        .expect("recorded");
    assert_eq!(message.task_ids, vec!["w1".to_owned(), "w2".to_owned()]);
}

#[test]
fn equivalent_speech_is_scoped_to_one_turn() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "acknowledgement",
            MessageRole::Assistant,
            "正在修改贪吃蛇，让它更酷炫！",
            MessageSource::RealtimeDirect,
        )
        .turn("turn-one"),
    );

    assert!(sync.has_equivalent_assistant_speech(
        &session,
        Some("turn-one"),
        "正在修改贪吃蛇，让它更酷炫。"
    ));
    assert!(
        !sync.has_equivalent_assistant_speech(
            &session,
            Some("turn-two"),
            "正在修改贪吃蛇，让它更酷炫。"
        ),
        "a later turn saying the same thing is a new statement"
    );
    assert!(!sync.has_equivalent_assistant_speech(
        &session,
        Some("turn-one"),
        "正在修改登录页面的颜色。"
    ));
}

#[test]
fn a_detailed_acknowledgement_is_recognised_as_the_same_action_preview() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "progress-preview",
            MessageRole::Assistant,
            "正在检查当前目录的项目进度。",
            MessageSource::RealtimeDirect,
        )
        .turn("turn-progress"),
    );

    assert!(sync.has_equivalent_assistant_speech(
        &session,
        Some("turn-progress"),
        "好的老大，我已经开始检查你当前这个项目的进度了，会看一下 git 分支、未提交改动和最近提交。"
    ));
}

#[test]
fn a_user_turn_is_never_matched_as_assistant_speech() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    sync.record(user("owner", "voice", "u1", "正在修改贪吃蛇，让它更酷炫！").turn("t"));
    assert!(!sync.has_equivalent_assistant_speech(
        &session,
        Some("t"),
        "正在修改贪吃蛇，让它更酷炫。"
    ));
}

#[test]
fn empty_or_punctuation_only_speech_is_never_equivalent() {
    let mut sync = ConversationSync::default();
    let session = SessionRef::new("owner", "voice");
    sync.record(
        RecordInput::new(
            "owner",
            "voice",
            "a",
            MessageRole::Assistant,
            "，。！",
            MessageSource::RealtimeDirect,
        )
        .turn("t"),
    );
    assert_eq!(speech_key("，。！"), "");
    assert!(!sync.has_equivalent_assistant_speech(&session, Some("t"), "，。！"));
    assert!(!sync.has_equivalent_assistant_speech(&session, Some("t"), "任何内容"));
}

#[test]
fn identical_short_speech_is_equivalent_but_similar_short_speech_is_not() {
    assert!(equivalent_speech("好的", "好的"));
    assert!(!equivalent_speech("好的", "好呀"));
    // Eight characters is the floor.
    assert!(!equivalent_speech("一二三四五六七", "一二三四五六八"));
    assert!(equivalent_speech("一二三四五六七八", "一二三四五六七八九"));
}

#[test]
fn messages_past_the_cap_are_dropped_oldest_first() {
    let mut sync = ConversationSync::new(Retention {
        max_messages: 3,
        ..Retention::default()
    });
    let session = SessionRef::new("owner", "voice");
    for index in 0..5 {
        sync.record(user(
            "owner",
            "voice",
            &format!("m{index}"),
            &format!("第{index}条"),
        ));
    }
    assert_eq!(
        contents(&sync.list(&session)),
        vec!["第2条".to_owned(), "第3条".to_owned(), "第4条".to_owned()]
    );
    assert_eq!(sync.sequence(), 5, "seq keeps counting past the cap");
}

#[test]
fn sessions_past_the_cap_are_evicted_least_recently_accessed_first() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let mut sync = ConversationSync::with_clock(
        Retention {
            max_sessions: 2,
            ..Retention::default()
        },
        Arc::new(move || *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())),
    );

    sync.record(user("owner", "s1", "a", "一"));
    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 2_000;
    sync.record(user("owner", "s2", "b", "二"));
    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 3_000;
    sync.record(user("owner", "s3", "c", "三"));

    assert_eq!(sync.session_count(), 2);
    assert!(sync.list(&SessionRef::new("owner", "s1")).is_empty());
    assert!(!sync.list(&SessionRef::new("owner", "s3")).is_empty());
}

#[test]
fn a_session_past_its_ttl_is_pruned_on_the_next_access() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let mut sync = ConversationSync::with_clock(
        Retention {
            session_ttl_ms: 100,
            ..Retention::default()
        },
        Arc::new(move || *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())),
    );
    let session = SessionRef::new("owner", "voice");
    sync.record(user("owner", "voice", "a", "内容"));
    assert_eq!(sync.list(&session).len(), 1);

    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 1_100;
    assert!(sync.list(&session).is_empty(), "the TTL is inclusive");
    assert_eq!(sync.session_count(), 0);
}

#[test]
fn a_read_keeps_a_session_alive() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let mut sync = ConversationSync::with_clock(
        Retention {
            session_ttl_ms: 100,
            ..Retention::default()
        },
        Arc::new(move || *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())),
    );
    let session = SessionRef::new("owner", "voice");
    sync.record(user("owner", "voice", "a", "内容"));

    for tick in [1_050i64, 1_100, 1_150] {
        *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = tick;
        assert_eq!(
            sync.list(&session).len(),
            1,
            "the read refreshes the session"
        );
    }
}

// ── the owning task ─────────────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn the_task_serializes_records_and_assigns_seq_in_order() {
    let (handle, task) = ConversationSyncHandle::spawn(ConversationSync::default());
    let session = SessionRef::new("owner", "voice");

    for index in 0..50u32 {
        handle
            .record(user(
                "owner",
                "voice",
                &format!("m{index}"),
                &format!("第{index}条"),
            ))
            .await
            .expect("the task is alive");
    }

    let messages = handle.list(&session).await.expect("listed");
    assert_eq!(messages.len(), 50);
    let sequence: Vec<u64> = messages.iter().map(|message| message.seq).collect();
    assert_eq!(
        sequence,
        (1..=50).collect::<Vec<u64>>(),
        "seq is monotonic and matches conversation order"
    );

    drop(handle);
    let recovered = task.await.expect("the task returns its state");
    assert_eq!(recovered.sequence(), 50);
}

#[tokio::test(start_paused = true)]
async fn concurrent_writers_never_share_a_seq() {
    let (handle, task) = ConversationSyncHandle::spawn(ConversationSync::default());
    let mut writers = Vec::new();
    for writer in 0..8u32 {
        let handle = handle.clone();
        writers.push(tokio::spawn(async move {
            for index in 0..10u32 {
                handle
                    .record(user(
                        "owner",
                        "voice",
                        &format!("w{writer}-{index}"),
                        &format!("{writer}/{index}"),
                    ))
                    .await
                    .expect("the task is alive");
            }
        }));
    }
    for writer in writers {
        writer.await.expect("the writer finished");
    }

    let messages = handle
        .list(&SessionRef::new("owner", "voice"))
        .await
        .expect("listed");
    let mut sequence: Vec<u64> = messages.iter().map(|message| message.seq).collect();
    sequence.sort_unstable();
    sequence.dedup();
    assert_eq!(
        sequence.len(),
        messages.len(),
        "80 records, 80 distinct sequence numbers"
    );
    // And the stored order agrees with the assigned order.
    let stored: Vec<u64> = messages.iter().map(|message| message.seq).collect();
    let mut ordered = stored.clone();
    ordered.sort_unstable();
    assert_eq!(stored, ordered);

    drop(handle);
    task.await.expect("the task ends when the last handle goes");
}

#[tokio::test(start_paused = true)]
async fn every_handle_method_reports_a_closed_task() {
    let (handle, task) = ConversationSyncHandle::spawn(ConversationSync::default());
    let session = SessionRef::new("owner", "voice");
    let orphan = handle.clone();
    drop(handle);
    // The task is still alive while `orphan` holds a sender.
    assert!(orphan.session_count().await.is_ok());
    drop(orphan);
    let recovered = task.await.expect("the task returns its state");
    let (handle, task) = ConversationSyncHandle::spawn(recovered);
    task.abort();
    // An aborted task cannot answer; every method says so rather than hanging
    // or inventing an answer.
    let _ = task.await;
    assert_eq!(
        handle.list(&session).await,
        Err(ConversationClosed),
        "a closed task is reported, not papered over"
    );
    assert_eq!(handle.session_count().await, Err(ConversationClosed));
    assert_eq!(handle.prune(0).await, Err(ConversationClosed));
}

#[tokio::test(start_paused = true)]
async fn retention_can_be_reconfigured_through_the_task() {
    let (handle, task) = ConversationSyncHandle::spawn(ConversationSync::default());
    handle
        .set_retention(Retention {
            max_messages: 2,
            ..Retention::default()
        })
        .await
        .expect("configured");
    for index in 0..4u32 {
        handle
            .record(user(
                "owner",
                "voice",
                &format!("m{index}"),
                &format!("第{index}条"),
            ))
            .await
            .expect("recorded");
    }
    let messages = handle
        .list(&SessionRef::new("owner", "voice"))
        .await
        .expect("listed");
    assert_eq!(
        contents(&messages),
        vec!["第2条".to_owned(), "第3条".to_owned()]
    );

    drop(handle);
    task.await.expect("done");
}
