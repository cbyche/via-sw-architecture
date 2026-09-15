//! [`MemoryExtractor`] — ported from `server/test/memory-extractor.test.mjs`
//! and `server/test/memory-extract-hook.test.mjs`, plus every gate in the
//! decision ladder.

use std::sync::Arc;

use pretty_assertions::assert_eq;
use via_conversation::audit::{AuditEvent, MemoryAudit, SkipReason};
use via_conversation::extractor::{
    DEFAULT_DEBOUNCE_MS, ExtractionOutcome, MemoryExtractor, MemoryExtractorBuilder,
    TranscriptSource, testing::ScriptedLlm, testing::VecTranscripts,
};
use via_conversation::markdown_store::MarkdownContextStore;
use via_conversation::memory_service::FrontendMemoryService;
use via_conversation::sync::{
    ConversationSync, ConversationSyncHandle, Message, MessageRole, MessageSource, RecordInput,
    SessionRef,
};
use via_i18n::Locale;

const OWNER: &str = "owner";
const SESSION: &str = "main";

struct Fixture {
    _dir: tempfile::TempDir,
    user: MarkdownContextStore,
    memory: MarkdownContextStore,
    extractor: MemoryExtractor,
    llm: Arc<ScriptedLlm>,
}

fn transcript(turns: usize, user_text: &str) -> Vec<Message> {
    (0..turns)
        .map(|index| Message {
            seq: index as u64 + 1,
            id: format!("user-{index}"),
            role: MessageRole::User,
            content: user_text.to_owned(),
            source: MessageSource::VoiceUser,
            turn_id: None,
            task_id: None,
            task_ids: Vec::new(),
            inputs: Vec::new(),
            created_at: 0,
        })
        .collect()
}

/// The audit's own file is the only record of what happened, so the tests read
/// it back rather than trusting a return value.
fn fixture_with(llm: Arc<ScriptedLlm>, turns: usize, user_text: &str, debounce_ms: i64) -> Fixture {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let user = MarkdownContextStore::builder()
        .file_path(dir.path().join("USER.md"))
        .scope("user")
        .template("# USER")
        .locale(Locale::Zh)
        .build();
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .locale(Locale::Zh)
        .build();
    let audit_path = dir.path().join("memory-audit.jsonl");
    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(Some(user.clone()), Some(memory.clone())),
        VecTranscripts::new(transcript(turns, user_text)),
    )
    .audit(
        MemoryAudit::builder()
            .file_path(&audit_path)
            .now(Arc::new(|| 1_000_000))
            .build(),
    )
    .llm(Some(llm.clone()))
    .locale(Locale::Zh)
    .now(Arc::new(|| 1_000_000))
    .debounce_ms(debounce_ms)
    .build();
    Fixture {
        _dir: dir,
        user,
        memory,
        extractor,
        llm,
    }
}

fn fixture(answer: &str) -> Fixture {
    fixture_with(ScriptedLlm::answering(answer), 4, "我每天早上都会跑步", 0)
}

fn audit_lines(fixture: &Fixture) -> Vec<serde_json::Value> {
    let path = fixture._dir.path().join("memory-audit.jsonl");
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("one JSON line"))
        .collect()
}

fn last_line(fixture: &Fixture) -> serde_json::Value {
    audit_lines(fixture)
        .pop()
        .expect("at least one audit line was written")
}

fn patch(document: &str, append: &str) -> String {
    serde_json::json!({
        "changes": [{ "document": document, "edits": [], "append": append }],
    })
    .to_string()
}

#[tokio::test(start_paused = true)]
async fn a_natural_markdown_patch_is_applied_to_memory() {
    let fixture = fixture(&patch("memory", "## 习惯\n\n- 用户每天早上跑步。"));
    let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;

    assert_eq!(outcome, ExtractionOutcome::Completed { changed: 1 });
    let document = fixture.memory.list(OWNER);
    assert!(document[0].content.contains("## 习惯"));
    assert!(document[0].content.contains("每天早上跑步"));

    let line = last_line(&fixture);
    assert_eq!(line["op"], serde_json::json!("patch"));
    assert_eq!(line["appended"], serde_json::json!(true));
    assert_eq!(line["edits"], serde_json::json!(0));
    assert_eq!(line["documents"], serde_json::json!(["memory"]));
    assert_eq!(line["beforeRevisions"]["memory"], serde_json::Value::Null);
    assert_eq!(
        line["afterRevisions"]["memory"],
        serde_json::json!(document[0].revision)
    );
    // The audit records what happened, never the memory text itself.
    assert!(!line.to_string().contains("每天早上跑步"));
}

#[tokio::test(start_paused = true)]
async fn an_exact_edit_corrects_existing_markdown() {
    let fixture = fixture(
        &serde_json::json!({
            "changes": [{
                "document": "memory",
                "edits": [{ "old_text": "每天晚上跑步", "new_text": "每天早上跑步" }],
                "append": "",
            }],
        })
        .to_string(),
    );
    fixture
        .memory
        .edit(
            OWNER,
            &via_conversation::EditRequest {
                append: "- 用户每天晚上跑步".to_owned(),
                ..via_conversation::EditRequest::default()
            },
        )
        .expect("seeded");

    let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;
    assert_eq!(outcome, ExtractionOutcome::Completed { changed: 1 });
    assert!(fixture.memory.read(OWNER).contains("每天早上跑步"));
    assert!(!fixture.memory.read(OWNER).contains("每天晚上跑步"));
    assert_eq!(last_line(&fixture)["edits"], serde_json::json!(1));
}

#[tokio::test(start_paused = true)]
async fn a_misplaced_directive_is_reconciled_across_both_documents() {
    let fixture = fixture_with(
        ScriptedLlm::answering(
            &serde_json::json!({
                "changes": [
                    {
                        "document": "user",
                        "edits": [],
                        "append": "- 每次回复都加一句“爱你哟”",
                    },
                    {
                        "document": "memory",
                        "edits": [{
                            "old_text": "- 用户要求助手每次回复都加一句“爱你哟”",
                            "new_text": "",
                        }],
                        "append": "",
                    },
                ],
            })
            .to_string(),
        ),
        4,
        "以后每次回复都加一句爱你哟",
        0,
    );
    fixture
        .memory
        .edit(
            OWNER,
            &via_conversation::EditRequest {
                append: "- 用户要求助手每次回复都加一句“爱你哟”".to_owned(),
                ..via_conversation::EditRequest::default()
            },
        )
        .expect("seeded");

    let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;
    assert_eq!(outcome, ExtractionOutcome::Completed { changed: 2 });
    assert!(fixture.user.read(OWNER).contains("爱你哟"));
    assert!(
        !fixture.memory.read(OWNER).contains("爱你哟"),
        "the directive left the factual document"
    );
}

#[tokio::test(start_paused = true)]
async fn explicit_interaction_directives_reach_user_md() {
    for (append, user_text) in [
        ("- 用户希望回答简洁一点", "以后回答简洁一点"),
        ("- 用户希望助手以后叫小舟", "你以后叫小舟"),
        (
            "- 用户要求助手在每次回复开头加上“爱你哟”",
            "你每次回复开头加上爱你哟",
        ),
        (
            "- 用户希望你在对话结尾说一句“你懂的”",
            "我希望你在对话结尾说一句你懂的",
        ),
    ] {
        let fixture = fixture_with(
            ScriptedLlm::answering(&patch("user", append)),
            4,
            user_text,
            0,
        );
        let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;
        assert_eq!(
            outcome,
            ExtractionOutcome::Completed { changed: 1 },
            "`{user_text}` should have authorized `{append}`"
        );
        assert!(fixture.user.read(OWNER).contains(&append[2..8]));
        assert_eq!(
            fixture.memory.read(OWNER),
            "",
            "a directive never lands in factual memory"
        );
        assert_eq!(last_line(&fixture)["op"], serde_json::json!("patch"));
    }
}

#[tokio::test(start_paused = true)]
async fn document_boundary_mistakes_and_secrets_are_refused() {
    for (document, append, reason) in [
        (
            "memory",
            "- 用户要求助手每次回复开头加上“爱你哟”",
            SkipReason::DocumentBoundary,
        ),
        ("user", "- 用户每天早上跑步", SkipReason::DocumentBoundary),
        ("memory", "- 用户的密码是 123456", SkipReason::Sensitive),
    ] {
        let fixture = fixture(&patch(document, append));
        let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;
        assert_eq!(
            outcome,
            ExtractionOutcome::Skipped(reason),
            "`{append}` in `{document}` should have been refused"
        );
        assert_eq!(fixture.user.read(OWNER), "");
        assert_eq!(fixture.memory.read(OWNER), "");
        let line = last_line(&fixture);
        assert_eq!(line["op"], serde_json::json!("skip"));
        assert_eq!(line["reason"], serde_json::json!(reason.as_str()));
    }
}

#[tokio::test(start_paused = true)]
async fn a_user_directive_is_never_inferred_without_transcript_evidence() {
    let fixture = fixture_with(
        ScriptedLlm::answering(&patch("user", "- 每次回复都说“爱你哟”")),
        4,
        "今天聊得很开心",
        0,
    );
    let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;

    assert_eq!(
        outcome,
        ExtractionOutcome::Skipped(SkipReason::UserDirectiveNotExplicit)
    );
    assert_eq!(fixture.user.read(OWNER), "");
    assert_eq!(
        last_line(&fixture)["reason"],
        serde_json::json!("user_directive_not_explicit")
    );
}

#[tokio::test(start_paused = true)]
async fn a_directive_in_the_assistants_own_turn_does_not_authorize_a_write() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let user = MarkdownContextStore::builder()
        .file_path(dir.path().join("USER.md"))
        .scope("user")
        .template("# USER")
        .build();
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .build();
    // Four user turns with nothing directive in them, and one assistant turn
    // that says the magic words. Only the user's side may authorize.
    let mut messages = transcript(4, "今天聊得很开心");
    messages.push(Message {
        seq: 5,
        id: "assistant".to_owned(),
        role: MessageRole::Assistant,
        content: "以后我每次回复都叫你老大".to_owned(),
        source: MessageSource::RealtimeDirect,
        turn_id: None,
        task_id: None,
        task_ids: Vec::new(),
        inputs: Vec::new(),
        created_at: 0,
    });

    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(Some(user.clone()), Some(memory)),
        VecTranscripts::new(messages),
    )
    .llm(Some(ScriptedLlm::answering(&patch(
        "user",
        "- 助手称呼用户：老大",
    ))))
    .locale(Locale::Zh)
    .build();

    assert_eq!(
        extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::Skipped(SkipReason::UserDirectiveNotExplicit)
    );
    assert_eq!(user.read(OWNER), "");
}

#[tokio::test(start_paused = true)]
async fn a_fenced_answer_and_a_trailing_comment_both_parse() {
    for answer in [
        "```json\n{\"changes\":[]}\n```",
        "{\"changes\":[]}\n记忆整理完成。",
    ] {
        let fixture = fixture(answer);
        assert_eq!(
            fixture.extractor.maybe_run(OWNER, SESSION).await,
            ExtractionOutcome::Skipped(SkipReason::NoChange)
        );
        assert_eq!(
            last_line(&fixture)["reason"],
            serde_json::json!("no_change")
        );
    }
}

#[tokio::test(start_paused = true)]
async fn an_invalid_change_is_refused_by_shape() {
    for answer in [
        // An unknown document — note that aliases are NOT resolved here.
        serde_json::json!({"changes":[{"document":"profile","append":"- x"}]}).to_string(),
        // The same document twice.
        serde_json::json!({"changes":[
            {"document":"memory","append":"- a"},
            {"document":"memory","append":"- b"},
        ]})
        .to_string(),
        // A change that proposes nothing at all.
        serde_json::json!({"changes":[{"document":"memory","edits":[],"append":""}]}).to_string(),
    ] {
        let fixture = fixture(&answer);
        assert_eq!(
            fixture.extractor.maybe_run(OWNER, SESSION).await,
            ExtractionOutcome::Skipped(SkipReason::InvalidChange),
            "answer: {answer}"
        );
        assert_eq!(fixture.memory.read(OWNER), "");
    }
}

#[tokio::test(start_paused = true)]
async fn assistant_md_is_unreachable_from_a_proposed_change() {
    let fixture = fixture(&patch("assistant", "## Identity\n\n你叫别的名字"));
    assert_eq!(
        fixture.extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::Skipped(SkipReason::InvalidChange),
        "the persona is not one of the two documents"
    );
    assert_eq!(fixture.user.read(OWNER), "");
    assert_eq!(fixture.memory.read(OWNER), "");
}

#[tokio::test(start_paused = true)]
async fn the_extractor_stays_disabled_without_a_model() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .build();
    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(None, Some(memory.clone())),
        VecTranscripts::new(transcript(4, "我每天早上都会跑步")),
    )
    .llm(None)
    .build();

    assert!(!extractor.enabled());
    let outcome = extractor.maybe_run(OWNER, SESSION).await;
    assert_eq!(outcome, ExtractionOutcome::Disabled);
    assert!(outcome.was_gated());
    assert_eq!(memory.read(OWNER), "");
}

#[tokio::test(start_paused = true)]
async fn a_quiet_session_and_a_missing_owner_are_gated_before_the_model() {
    let quiet = fixture_with(
        ScriptedLlm::answering("{\"changes\":[]}"),
        2,
        "我每天早上都会跑步",
        0,
    );
    assert_eq!(
        quiet.extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::TooQuiet
    );
    assert_eq!(quiet.llm.call_count(), 0, "a gated run costs nothing");

    let anonymous = fixture("{\"changes\":[]}");
    assert_eq!(
        anonymous.extractor.maybe_run("", SESSION).await,
        ExtractionOutcome::NoOwner
    );
    assert_eq!(anonymous.llm.call_count(), 0);
}

#[tokio::test(start_paused = true)]
async fn a_second_close_inside_the_window_is_debounced() {
    let fixture = fixture_with(
        ScriptedLlm::answering("{\"changes\":[]}"),
        4,
        "我每天早上都会跑步",
        DEFAULT_DEBOUNCE_MS,
    );
    assert_eq!(
        fixture.extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::Skipped(SkipReason::NoChange),
        "the first close always qualifies"
    );
    assert_eq!(
        fixture.extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::Debounced
    );
    assert_eq!(fixture.llm.call_count(), 1);
}

#[tokio::test(start_paused = true)]
async fn concurrent_closes_claim_the_window_once() {
    let fixture = fixture_with(
        ScriptedLlm::answering("{\"changes\":[]}"),
        4,
        "我每天早上都会跑步",
        DEFAULT_DEBOUNCE_MS,
    );
    let extractor = fixture.extractor.clone();
    let (first, second) = tokio::join!(
        extractor.maybe_run(OWNER, SESSION),
        fixture.extractor.maybe_run(OWNER, SESSION),
    );
    let outcomes = [first, second];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| **outcome == ExtractionOutcome::Debounced)
            .count(),
        1,
        "exactly one of the two closes is debounced: {outcomes:?}"
    );
    assert_eq!(fixture.llm.call_count(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_debounced_owner_does_not_block_another_owner() {
    let fixture = fixture_with(
        ScriptedLlm::answering("{\"changes\":[]}"),
        4,
        "我每天早上都会跑步",
        DEFAULT_DEBOUNCE_MS,
    );
    assert!(
        !fixture
            .extractor
            .maybe_run("owner-a", SESSION)
            .await
            .was_gated()
    );
    assert_eq!(
        fixture.extractor.maybe_run("owner-a", SESSION).await,
        ExtractionOutcome::Debounced
    );
    assert!(
        !fixture
            .extractor
            .maybe_run("owner-b", SESSION)
            .await
            .was_gated()
    );
}

#[tokio::test(start_paused = true)]
async fn a_provider_failure_and_malformed_output_are_recorded_not_raised() {
    for llm in [
        ScriptedLlm::failing("provider unavailable"),
        ScriptedLlm::answering("not json"),
        ScriptedLlm::answering("{}"),
    ] {
        let fixture = fixture_with(llm, 4, "我每天早上都会跑步", 0);
        let outcome = fixture.extractor.maybe_run(OWNER, SESSION).await;
        assert!(
            matches!(outcome, ExtractionOutcome::Failed(_)),
            "expected a recorded failure, got {outcome:?}"
        );
        assert_eq!(last_line(&fixture)["op"], serde_json::json!("error"));
        assert_eq!(fixture.memory.read(OWNER), "");
    }
}

#[tokio::test(start_paused = true)]
async fn an_empty_transcript_never_reaches_the_model() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .build();
    let llm = ScriptedLlm::answering("{\"changes\":[]}");
    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(None, Some(memory)),
        VecTranscripts::new(Vec::new()),
    )
    .llm(Some(llm.clone()))
    .min_user_messages(0)
    .build();

    assert_eq!(
        extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::EmptyTranscript
    );
    assert_eq!(llm.call_count(), 0);
}

#[tokio::test(start_paused = true)]
async fn the_model_is_shown_the_current_documents_and_the_transcript() {
    let fixture = fixture("{\"changes\":[]}");
    fixture
        .memory
        .edit(
            OWNER,
            &via_conversation::EditRequest {
                append: "- 已有的事实".to_owned(),
                ..via_conversation::EditRequest::default()
            },
        )
        .expect("seeded");

    fixture.extractor.maybe_run(OWNER, SESSION).await;
    let calls = fixture.llm.calls();
    assert_eq!(calls.len(), 1);
    let (system, user) = &calls[0];
    assert!(system.contains("USER.md") && system.contains("MEMORY.md"));
    assert!(user.contains("## 当前 USER.md"));
    assert!(
        user.contains("# USER"),
        "an absent document shows its heading"
    );
    assert!(user.contains("- 已有的事实"), "so the model can correct it");
    assert!(user.contains("## 对话转写"));
    assert!(user.contains("用户: 我每天早上都会跑步"));
}

#[tokio::test(start_paused = true)]
async fn the_transcript_keeps_the_newest_turns_within_its_budget() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .build();
    let messages: Vec<Message> = (0..20)
        .map(|index| Message {
            seq: index as u64 + 1,
            id: format!("m{index}"),
            role: MessageRole::User,
            content: format!("第{index:02}条{}", "字".repeat(50)),
            source: MessageSource::VoiceUser,
            turn_id: None,
            task_id: None,
            task_ids: Vec::new(),
            inputs: Vec::new(),
            created_at: 0,
        })
        .collect();
    let llm = ScriptedLlm::answering("{\"changes\":[]}");
    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(None, Some(memory)),
        VecTranscripts::new(messages),
    )
    .llm(Some(llm.clone()))
    .locale(Locale::Zh)
    .max_transcript_chars(200)
    .build();

    extractor.maybe_run(OWNER, SESSION).await;
    let (_, user) = llm.calls().pop().expect("one call");
    assert!(user.contains("第19条"), "the newest turn survives");
    assert!(!user.contains("第00条"), "the oldest does not");
}

/// An [`ExtractorLlm`] that writes to a document while it is "thinking",
/// reproducing a realtime `memory` tool call that lands during the extraction
/// request.
#[derive(Debug)]
struct RacingLlm {
    store: MarkdownContextStore,
    answer: String,
}

#[async_trait::async_trait]
impl via_conversation::ExtractorLlm for RacingLlm {
    async fn complete(
        &self,
        _system: &str,
        _user: &str,
    ) -> Result<String, via_conversation::ExtractorError> {
        self.store
            .edit(
                OWNER,
                &via_conversation::EditRequest {
                    append: "- 用户刚说出口的内容".to_owned(),
                    ..via_conversation::EditRequest::default()
                },
            )
            .expect("the realtime tool writes while the model thinks");
        Ok(self.answer.clone())
    }
}

#[tokio::test(start_paused = true)]
async fn a_realtime_write_during_the_model_call_makes_the_patch_stale() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .locale(Locale::Zh)
        .build();
    memory
        .edit(
            OWNER,
            &via_conversation::EditRequest {
                append: "- 旧内容".to_owned(),
                ..via_conversation::EditRequest::default()
            },
        )
        .expect("seeded");
    let audit_path = dir.path().join("memory-audit.jsonl");

    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(None, Some(memory.clone())),
        VecTranscripts::new(transcript(4, "我每天早上都会跑步")),
    )
    .audit(MemoryAudit::builder().file_path(&audit_path).build())
    .llm(Some(Arc::new(RacingLlm {
        store: memory.clone(),
        answer: serde_json::json!({
            "changes": [{
                "document": "memory",
                "edits": [{ "old_text": "- 旧内容", "new_text": "- 新内容" }],
                "append": "",
            }],
        })
        .to_string(),
    })))
    .locale(Locale::Zh)
    .build();

    let outcome = extractor.maybe_run(OWNER, SESSION).await;

    // The revision the extractor was shown moved under it, so the patch is
    // refused rather than written over what the user just said out loud.
    assert!(
        matches!(outcome, ExtractionOutcome::Failed(_)),
        "expected a refusal, got {outcome:?}"
    );
    let document = memory.read(OWNER);
    assert!(document.contains("- 旧内容"), "nothing was replaced");
    assert!(
        document.contains("- 用户刚说出口的内容"),
        "the live write stands"
    );
    assert!(!document.contains("- 新内容"));

    let line: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(&audit_path)
            .expect("readable")
            .lines()
            .next_back()
            .expect("one line"),
    )
    .expect("json");
    assert_eq!(line["op"], serde_json::json!("error"));
    assert!(
        line["error"]
            .as_str()
            .unwrap_or_default()
            .contains("read it again"),
        "the audit says why: {line}"
    );
}

#[tokio::test(start_paused = true)]
async fn the_conversation_task_is_a_transcript_source() {
    let (handle, task) = ConversationSyncHandle::spawn(ConversationSync::default());
    for index in 0..4u32 {
        handle
            .record(RecordInput::new(
                OWNER,
                SESSION,
                format!("m{index}"),
                MessageRole::User,
                "以后回答简洁一点",
                MessageSource::VoiceUser,
            ))
            .await
            .expect("recorded");
    }

    let messages = TranscriptSource::list(&handle, OWNER, SESSION).await;
    assert_eq!(messages.len(), 4);
    assert_eq!(
        handle
            .list(&SessionRef::new(OWNER, SESSION))
            .await
            .expect("listed")
            .len(),
        4
    );

    drop(handle);
    task.await.expect("done");
}

#[tokio::test(start_paused = true)]
async fn a_run_with_no_audit_configured_still_completes() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .build();
    let extractor = MemoryExtractorBuilder::new(
        FrontendMemoryService::new(None, Some(memory.clone())),
        VecTranscripts::new(transcript(4, "我每天早上都会跑步")),
    )
    .llm(Some(ScriptedLlm::answering(&patch(
        "memory",
        "- 用户每天早上跑步。",
    ))))
    .build();

    assert_eq!(
        extractor.maybe_run(OWNER, SESSION).await,
        ExtractionOutcome::Completed { changed: 1 }
    );
    assert!(memory.read(OWNER).contains("每天早上跑步"));
}

#[test]
fn an_audit_event_names_its_op_first() {
    let value =
        serde_json::to_value(AuditEvent::skip("owner", SkipReason::Sensitive)).expect("json");
    assert_eq!(
        value.as_object().expect("an object").keys().next(),
        Some(&"op".to_owned())
    );
}
