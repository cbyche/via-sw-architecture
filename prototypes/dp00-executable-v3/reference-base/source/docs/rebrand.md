# Rebrand: qwen-audio-agent → VIA

Generated from a full survey of upstream v1.11.0 (2026-08-22). The rule is
**no qwen, except the model and the frontend voice engine's qwen runtime.**

Two categories, 198 entries.

- **RENAME** (133) — strings that carry *our* product identity.
- **KEEP** (65) — strings owned by someone else: vendor model ids, third-party
  products VIA integrates with, external protocol method names, and brand-free
  filenames. Renaming any of these breaks something real.

> Watch two entries in particular: the five MCP tool names (the permission
> broker's auto-approve allowlist matches them three different ways) and the
> wake word (changing it requires a regenerated sherpa-onnx keyword model).
> See [`architecture.md` §8](architecture.md).

## RENAME — 133

| Upstream | VIA | Where |
| --- | --- | --- |
| `${QWEN_AUDIO_AGENT_OPENCLAW_MODEL_ID}` | `${VIA_OPENCLAW_MODEL_ID}` | config/openclaw/openclaw.json5:11 and :12 (models[0].id and models[0].name); set at scripts/openclaw.mjs:79; … |
| `${QWEN_AUDIO_AGENT_OPENCLAW_MODEL}` | `${VIA_OPENCLAW_MODEL}` | config/openclaw/openclaw.json5:27 and :28 (agents.defaults.model.primary and the models map key); set at scri… |
| `${QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE}` | `${VIA_OPENCLAW_WORKSPACE}` | config/openclaw/openclaw.json5:35 (agents.list[0].workspace); scripts/openclaw.mjs:52-54; shared/backend-cata… |
| `'# qwen-audio-agent 用户配置' (config.env template header)` | `'# VIA 用户配置'` | shared/runtime-environment.mjs:22 |
| `'A qwenaudio-owned Gateway only provides the local ACP backend.' (comment) and the tmp-fi…` | `via-owned / 'via-memory-e2e-', 'via-smoke-', 'via-install-'` | server/src/process/backend-drivers/openclaw-auth.mjs:83-84; test and installer helpers |
| `'qwen-audio-agent 已在运行：<url>' / '已就绪' / '启动失败' (scripts/start.mjs stdout)` | `'VIA 已在运行：…' etc.` | scripts/start.mjs:25,27,41 |
| `'qwen-audio-agent 日志写入失败：<message>' (stderr on log sink failure) and 'qwen-audio-agent ru…` | `'VIA 日志写入失败：…' and 'VIA running at <origin>'` | shared/logger.mjs:280; server/src/app/gateway-application.mjs:504 |
| `'你好千问' (config.wakeWord)` | `a VIA wake phrase (product decision), with a matching sherpa-onnx keyword model…` | server/src/core/config.mjs:503; sent to clients in every voice.sleep event and in the '已休眠，请先说“…”唤醒。' error (… |
| `<qwen_audio_agent_backend_instructions> / </qwen_audio_agent_backend_instructions>` | `<via_backend_instructions> / </via_backend_instructions>` | server/src/agent/acp-backend-adapter.mjs:949,952 — coordinatorInstructions wrapper on every coordinator prompt |
| `<qwen_audio_agent_backend_instructions> / <qwen_audio_agent_reconciliation> / <qwen_audio…` | `<via_backend_instructions> / <via_reconciliation> / <via_delegation_result> / <…` | server/src/agent/acp-backend-adapter.mjs:949,952,1010,1012,1141,1149,1391,1394,1436,1442 |
| `<qwen_audio_agent_control kind="cancel"> / kind="status"> and </qwen_audio_agent_control>` | `<via_control kind="cancel"> / <via_control kind="status"> / </via_control>` | server/src/agent/acp-backend-adapter.mjs:1391,1394,1436,1442 — hidden control turns |
| `<qwen_audio_agent_delegation_result> / </qwen_audio_agent_delegation_result>` | `<via_delegation_result> / </via_delegation_result>` | server/src/agent/acp-backend-adapter.mjs:1141,1149 — delegationResultPrompt |
| `<qwen_audio_agent_progress> ... </qwen_audio_agent_progress>` | `<via_progress> ... </via_progress>` | server/src/voice/realtime-gateway.mjs:843,847 — XML-ish tag wrapping background task progress text injected i… |
| `<qwen_audio_agent_protocol_retry> …` | `<via_protocol_retry>` | server/src/agent/coordinator.mjs:271,275 |
| `<qwen_audio_agent_reconciliation> / </qwen_audio_agent_reconciliation>` | `<via_reconciliation> / </via_reconciliation>` | server/src/agent/acp-backend-adapter.mjs:1010,1012 — pending-facts injection block |
| `<qwen_audio_agent_request> / </qwen_audio_agent_request>` | `<via_request> / </via_request>` | server/src/agent/coordinator.mjs:210,212 |
| `<qwen_audio_agent_request> / <qwen_audio_agent_protocol_retry>` | `<via_request> / <via_protocol_retry>` | server/src/agent/coordinator.mjs:210,212,271,275 (adjacent scope) — the envelope and retry prompt tags carrie… |
| `<qwen_audio_agent_work_results> … </qwen_audio_agent_work_results>` | `<via_work_results> … </via_work_results>` | server/src/voice/announcement/announcement-manager.mjs:387,390 — XML-ish wrapper tag inside the model-visible… |
| `@qwen-audio-agent/cli` | `via-cli (Cargo package)` | cli/package.json:2; package.json:157 (`npm run cli`), 167 (test script) |
| `@qwen-audio-agent/server \| /web \| /tui \| /desktop \| /cli` | `via-core / via-voice / via-agent / via-task / via-app / via-cli (Cargo crates)` | package.json workspaces; test/set-version.test.mjs:12,31 |
| `agent:${coordinatorAgent}:qwen-audio-agent:${owner}:backend` | `agent:${coordinatorAgent}:via:${owner}:backend` | server/src/agent/backends/openclaw.mjs:114-117 — coordinatorMeta().sessionKey, sent over the wire in session/… |
| `agent:<coordinatorAgent>:qwen-audio-agent:<owner>:backend` | `agent:<coordinatorAgent>:via:<owner>:backend` | server/src/agent/backends/openclaw.mjs:113-117 — coordinatorMeta sessionKey template |
| `agent:voice-coordinator:qwen-audio-agent:<owner>:backend` | `agent:voice-coordinator:via:<owner>:backend` | OpenClaw ACP sessionKey; server/test/acp-backend-adapter.test.mjs:1900 |
| `cli/bin/qwenaudio.mjs` | `crates/via-cli/src/main.rs producing the `via` binary` | file path; package.json:50,53; scripts/verify-package.mjs:111 required-files list |
| `CODEX_PROVIDER = 'qwen-audio-agent'` | `via` | server/src/agent/backends/codex.mjs:4 — written into MODEL_PROVIDER and as the CODEX_CONFIG model_provider ke… |
| `com.qwen-audio-agent.gateway` | `com.via.gateway` | cli/src/gateway-service.mjs:11 (GATEWAY_SERVICE_LABEL); launchd plist filename; launchctl domain target gui/<… |
| `Description=qwen-audio-agent Gateway` | `Description=VIA Gateway` | cli/src/gateway-service.mjs:126 |
| `https://github.com/QwenAudio/qwen-audio-agent` | `the VIA repository URL` | package.json repository/bugs/homepage; README badges updated by test/set-version.test.mjs:44-49,89-94 |
| `https://github.com/QwenAudio/qwen-audio-agent(.git) — repository, bugs, homepage, and the…` | `the VIA repository URL` | package.json:10,13,15; docs/getting-started/install.md:20,32,44 |
| `keywords: ["qwen", "audio", "voice", "agent", …]` | `drop the npm keywords block entirely (Cargo uses `keywords` with at most 5 entr…` | package.json:16-27 |
| `model_providers['qwen-audio-agent'] in CODEX_CONFIG` | `model_providers['via']` | server/test/acp-backend-adapter.test.mjs:744 |
| `qwaudio (default config directory name)` | `via` | shared/runtime-environment.mjs:100 — resolve(base, 'qwaudio'); server/test/config.test.mjs:29,61,69 use '/hom… |
| `qwaudio (final path segment of ~/.config/qwaudio and ~/.config/qwaudio/logs)` | `via (~/.config/via, ~/.config/via/logs)` | shared/runtime-environment.mjs:100; shared/logger.mjs:56 |
| `qwaudio-* / qwen-audio-agent-* / qwen-audio-version-* tmpdir prefixes` | `via-*` | test/{gateway-instance-lock,logger,set-version,consumer-install}.test.mjs; server/test/{acp-session-registry,… |
| `qwaudio.gateway-lock/v1` | `via.gateway-lock/v1` | shared/gateway-instance-lock.mjs:13 (GATEWAY_LOCK_SCHEMA), written into <configDir>/gateway.lock and validate… |
| `qwaudio.gateway-lock/v1 and qwaudio.log/v1` | `via.gateway-lock/v1 and via.log/v1` | shared/gateway-instance-lock.mjs:13; shared/logger.mjs:13 |
| `qwaudio.log/v1` | `via.log/v1` | shared/logger.mjs:13 (LOG_SCHEMA), emitted on every log line |
| `QWAUDIO_CONFIG_DIR` | `VIA_CONFIG_DIR` | shared/backend-catalog.mjs:69 (forwarded to the openclaw child); also test/consumer-install.test.mjs and the … |
| `QWAUDIO_CONFIG_DIR / QWAUDIO_DATA_DIR` | `VIA_CONFIG_DIR / VIA_DATA_DIR` | shared/runtime-environment.mjs:96, 111 — resolve configDirectory and dataDirectory, which locate tasks.json, … |
| `QWAUDIO_CONFIG_DIR, QWAUDIO_DATA_DIR` | `VIA_CONFIG_DIR, VIA_DATA_DIR` | shared/runtime-environment.mjs:96,111; shared/logger.mjs:52 |
| `QWAUDIO_DATA_DIR` | `VIA_DATA_DIR` | shared/runtime-environment.mjs:111 |
| `QWAUDIO_GATEWAY_ALREADY_RUNNING` | `VIA_GATEWAY_ALREADY_RUNNING` | shared/gateway-instance-lock.mjs:157 |
| `QWAUDIO_GATEWAY_ALREADY_RUNNING, QWAUDIO_GATEWAY_SETUP_REQUIRED, QWAUDIO_INPUT_OWNER_REQU…` | `VIA_GATEWAY_ALREADY_RUNNING, VIA_GATEWAY_SETUP_REQUIRED, VIA_INPUT_OWNER_REQUIR…` | shared/gateway-instance-lock.mjs:157; shared/gateway-setup.mjs:49; server/src/voice/input-arbitration.mjs:73 … |
| `QWAUDIO_GATEWAY_SETUP_REQUIRED` | `VIA_GATEWAY_SETUP_REQUIRED` | shared/gateway-setup.mjs:49 (error.code) |
| `QWAUDIO_GATEWAY_SETUP_REQUIRED / qwaudio.gateway-lock/v1` | `VIA_GATEWAY_SETUP_REQUIRED / via.gateway-lock/v1` | docs/contract.md:33,152 — setup-gate error code and the gateway lease file schema string written into gateway… |
| `QWAUDIO_GATEWAY_SETUP_REQUIRED, QWAUDIO_GATEWAY_ALREADY_RUNNING, QWAUDIO_INPUT_OWNER_REQU…` | `VIA_GATEWAY_SETUP_REQUIRED, VIA_GATEWAY_ALREADY_RUNNING, VIA_INPUT_OWNER_REQUIR…` | shared/gateway-setup.mjs:49, shared/gateway-instance-lock.mjs, server/src/voice/input-arbitration.mjs |
| `QWAUDIO_INPUT_OWNER_REQUIRED` | `VIA_INPUT_OWNER_REQUIRED` | server/src/voice/input-arbitration.mjs:73 (Error.code) and server/src/app/gateway-application.mjs:283-284 (HT… |
| `qwen(?!-audio-agent) inside the named-backend ban regex` | `the regex becomes /\b(?:openclaw\|opencode\|qoder\|qwen\|kimi\|hermes\|codebudd…` | server/test/dependency-boundaries.test.mjs:68 |
| `qwen-audio-agent` | `via` | server/src/agent/backend-agent-instructions.mjs:2,5,14 — three sentences of the model-visible backend instruc… |
| `qwen-audio-agent (ACP client identity)` | `via / 'VIA Gateway'` | server/src/agent/acp-process-client.mjs:173 acp.client({name:'qwen-audio-agent'}) and :219-220 clientInfo {na… |
| `qwen-audio-agent (npm package name), @qwen-audio-agent/server, @qwen-audio-agent/cli, qwe…` | `via (crate/package), via-server, via-cli, via.desktop` | package.json:2-3,6,10,13,15; server/package.json:2; package.json:157,167 |
| `qwen-audio-agent (product name inside strings)` | `via` | shared/logger.mjs:280 (stderr write-failure prefix); shared/runtime-environment.mjs:22 (config.env header com… |
| `qwen-audio-agent Backend Agent` | `VIA Backend Agent` | config/openclaw/openclaw.json5:34 — agents.list[0].name |
| `qwen-audio-agent Gateway` | `VIA Gateway` | server/src/agent/acp-process-client.mjs:220 — initialize params clientInfo.title |
| `qwen-audio-agent 重启时这项工作尚未完成，请重新提交。` | `VIA 重启时这项工作尚未完成，请重新提交。` | server/src/task/task-manager.mjs:194 — restore() error string written into task.error, persisted to tasks.jso… |
| `qwen-audio-agent 重启时这项项目任务失去连接，请重新提交。` | `VIA 重启时这项项目任务失去连接，请重新提交。` | server/src/task/task-manager.mjs:262 — recoverDelegated() error string for unrecoverable delegated work |
| `qwen-audio-agent-backend` | `via-backend` | OpenCode default coordinator-agent sentinel; server/test/config.test.mjs:49, server/test/acp-backend-adapter.… |
| `qwen-audio-agent-backend, qwen-audio-agent-coordinator` | `via-backend, via-coordinator` | server/src/process/backend-drivers/opencode.mjs:27-29 (reserved agent names that configuredAgent() blanks); s… |
| `qwen-audio-agent-gateway.service` | `via-gateway.service` | cli/src/gateway-service.mjs:123,307,316; unit path $XDG_CONFIG_HOME/systemd/user/ |
| `qwen-audio-agent.coordination.v1` | `via.coordination.v1` | server/src/agent/coordinator.mjs:181 — envelope `protocol` field |
| `qwen-audio-agent.desktop` | `via.desktop` | package.json desktopName:3 |
| `qwen-audio-agent.desktop / qwen-audio-agent-*-architecture.png\|svg` | `via.desktop / via-*-architecture.*` | package.json:3 desktopName, :66-69 files list |
| `qwen-audio-agent/inputRef` | `via/inputRef` | shared/input-parts.mjs:4 — INPUT_REF_META_KEY, written into ContentBlock._meta and read back by the voice lay… |
| `qwen-audio-agent://input/` | `via://input/` | server/src/agent/acp-content.mjs:8 — resourceUri(); becomes ContentBlock.uri for images and resource.uri for … |
| `qwen-audio-agent://input/<filename>` | `via://input/<filename>` | server/src/agent/acp-content.mjs:8 — URI scheme for prompt attachments |
| `qwen-audio-agent:gateway-ready` | `via:gateway-ready` | shared/gateway-process.mjs:17 (GATEWAY_READY_MESSAGE), emitted at server/src/app/gateway-application.mjs:489 |
| `qwen-audio-agent:offline-notification` | `via:offline-notification` | server/src/app/offline-notifications.mjs:20 and :32 — parentPort IPC message `type` consumed by the Electron … |
| `QWEN_AUDIO_* (env prefix, non-AGENT variants)` | `VIA_* — but PRESERVE the Audio/Omni family distinction: QWEN_AUDIO_REALTIME_VOI…` | .env.example:8,11,12,17 (QWEN_AUDIO_REALTIME_MODEL, QWEN_AUDIO_REALTIME_VOICE, QWEN_OMNI_REALTIME_VOICE, QWEN… |
| `QWEN_AUDIO_* (env prefix: LOG_*, MEMORY_*, WAKE_WORD_*, GATEWAY_*, DESKTOP_AUTO_HIDE_SECO…` | `VIA_LOG_*, VIA_MEMORY_*, VIA_WAKE_WORD_*, VIA_GATEWAY_*, VIA_AUTO_HIDE_SECONDS,…` | shared/logger.mjs:51,248,256-257,266,272; server/src/core/config.mjs:448,450,453,456,496,501,504; shared/gate… |
| `QWEN_AUDIO_* env prefix (QWEN_AUDIO_REALTIME_MODEL, QWEN_AUDIO_REALTIME_BASE_URL, QWEN_AU…` | `VIA_* (VIA_REALTIME_MODEL, VIA_REALTIME_BASE_URL, VIA_REALTIME_API_KEY, VIA_REA…` | .env.example; shared/logger.mjs:266-273; shared/gateway-setup.mjs:41; server/src/app/gateway-application.mjs:… |
| `qwen_audio_agent` | `via` | server/src/agent/acp-session-tools.mjs:9 (ACP_SESSION_TOOL_SERVER), 186 (descriptor name), 232 (McpServer nam… |
| `QWEN_AUDIO_AGENT_*` | `VIA_* (e.g. VIA_URL, VIA_SESSION_ID, VIA_BACKEND_PERMISSION_MODE, VIA_OPENCLAW_…` | cli/src/arguments.mjs:103,104,106,110,113; cli/src/launcher.mjs:68,72,79,183; cli/src/runtime.mjs:73,77,103,2… |
| `QWEN_AUDIO_AGENT_* (env prefix)` | `VIA_* (e.g. VIA_TASK_STATE_PATH, VIA_REMINDER_STAGGER_MS, VIA_FRONTEND_NOTES_PA…` | All Work/notes/memory settings read by my modules via config.mjs: QWEN_AUDIO_AGENT_TASK_STATE_PATH, _TASK_TER… |
| `QWEN_AUDIO_AGENT_* (env prefix, 45+ variables)` | `VIA_* (e.g. VIA_AUTH_SECRET, VIA_ALLOWED_ORIGINS, VIA_BACKEND_MODEL, VIA_RUNTIM…` | server/src/core/config.mjs (throughout), server/src/index.mjs:12,53,60, server/src/process/managed-backend.mj… |
| `QWEN_AUDIO_AGENT_* (env prefix, ~40+ names)` | `VIA_* (e.g. VIA_BACKEND_PERMISSION_MODE, VIA_BACKEND_MODEL, VIA_ALLOWED_ORIGINS…` | .env.example:29,30,33,99 (BACKEND_PERMISSION_MODE, BACKEND_AGENT, BACKEND_MODEL, ALLOWED_ORIGINS); plus serve… |
| `QWEN_AUDIO_AGENT_* env prefix (QWEN_AUDIO_AGENT_AUTH_SECRET, _ALLOWED_ORIGINS, _BACKEND_M…` | `VIA_* (VIA_AUTH_SECRET, VIA_ALLOWED_ORIGINS, VIA_BACKEND_MODEL, VIA_BACKEND_AGE…` | .env.example:29-99; server/test/{managed-backend,backend-install,backend-lifecycle,backend-onboarding,builtin… |
| `QWEN_AUDIO_AGENT_ACP_FORWARD_ENV` | `VIA_ACP_FORWARD_ENV` | shared/backend-catalog.mjs:387 (acp explicitListEnvironment); shared/runtime-environment.mjs:40 (config templ… |
| `QWEN_AUDIO_AGENT_ANNOUNCE_INTO_CONTEXT, QWEN_AUDIO_AGENT_RESULT_CONTEXT_MAX_CHARS, QWEN_A…` | `VIA_ANNOUNCE_INTO_CONTEXT, VIA_RESULT_CONTEXT_MAX_CHARS, VIA_ANNOUNCEMENT_BATCH…` | server/src/core/config.mjs:334-364 — env vars that tune the announcement engine in this scope |
| `QWEN_AUDIO_AGENT_AUTH_SECRET` | `VIA_AUTH_SECRET` | shared/runtime-environment.mjs:20 (SECRET_KEY), persisted in state.env |
| `QWEN_AUDIO_AGENT_BACKEND_AGENT, QWEN_AUDIO_AGENT_BACKEND_MODEL, QWEN_AUDIO_AGENT_BACKEND_…` | `VIA_BACKEND_AGENT, VIA_BACKEND_MODEL, VIA_BACKEND_OWNERSHIP, VIA_BACKEND_PERMIS…` | shared/backend-environment.mjs:49-61 (INTERNAL_NAMES — the variables that DO cross into every child agent), a… |
| `QWEN_AUDIO_AGENT_BACKEND_AGENT, QWEN_AUDIO_AGENT_BACKEND_MODEL, QWEN_AUDIO_AGENT_BACKEND_…` | `VIA_BACKEND_AGENT, VIA_BACKEND_MODEL, VIA_BACKEND_OWNERSHIP, VIA_BACKEND_PERMIS…` | shared/backend-environment.mjs:49-61 — INTERNAL_NAMES, the allowlist forwarded into every spawned backend pro… |
| `QWEN_AUDIO_AGENT_BACKEND_MODEL, _BACKEND_AGENT, _BACKEND_OWNERSHIP, _BACKEND_PERMISSION_M…` | `VIA_BACKEND_MODEL, VIA_BACKEND_AGENT, VIA_BACKEND_OWNERSHIP, VIA_BACKEND_PERMIS…` | shared/backend-environment.mjs:50-53; shared/backend-onboarding.mjs:8; shared/backend-setup.mjs:367,368,404,4… |
| `QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH` | `VIA_BACKEND_SESSION_STATE_PATH` | server/src/core/config.mjs:389 — path override for state/acp-sessions.json |
| `QWEN_AUDIO_AGENT_COMPUTER_USE` | `VIA_COMPUTER_USE` | server/src/agent/builtin-mcp.mjs:54 — feature toggle env var |
| `qwen_audio_agent_delegation_result, qwen_audio_agent_reconciliation` | `via_delegation_result, via_reconciliation` | coordinator prompt tokens; server/test/acp-backend-adapter.test.mjs:185,1379 |
| `QWEN_AUDIO_AGENT_DESKTOP, QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY` | `VIA_DESKTOP, VIA_DESKTOP_INSTALLED_ONLY` | shared/backend-environment.mjs:54,55; shared/backend-setup.mjs:285,287,371 |
| `QWEN_AUDIO_AGENT_ENV_LOADED, _NODE, _ROOT, _RUNTIME_ROOT, _SOURCE_ROOT` | `VIA_ENV_LOADED, VIA_NODE, VIA_ROOT, VIA_RUNTIME_ROOT, VIA_SOURCE_ROOT` | shared/backend-environment.mjs:56-60,99-100; server/src/index.mjs:12 |
| `qwen_audio_agent_identity` | `via_identity` | server/src/core/identity.mjs:7 (COOKIE_NAME) |
| `QWEN_AUDIO_AGENT_OPENCLAW_STATE_DIR` | `VIA_OPENCLAW_STATE_DIR` | scripts/openclaw.mjs:51,53; shared/runtime-environment.mjs:558 |
| `QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE, _OPENCLAW_STATE_DIR, _OPENCLAW_MODEL, _OPENCLAW_MODE…` | `VIA_OPENCLAW_WORKSPACE, VIA_OPENCLAW_STATE_DIR, VIA_OPENCLAW_MODEL, VIA_OPENCLA…` | shared/backend-catalog.mjs:42,70-73; shared/runtime-environment.mjs:519,520,558 |
| `QWEN_AUDIO_AGENT_OPENCODE_ISOLATE_USER_CONFIG, _OPENCODE_XDG_CONFIG_HOME` | `VIA_OPENCODE_ISOLATE_USER_CONFIG, VIA_OPENCODE_XDG_CONFIG_HOME` | shared/backend-catalog.mjs:33-34 |
| `QWEN_AUDIO_AGENT_OPENCODE_ISOLATE_USER_CONFIG, QWEN_AUDIO_AGENT_OPENCODE_XDG_CONFIG_HOME,…` | `VIA_OPENCODE_ISOLATE_USER_CONFIG, VIA_OPENCODE_XDG_CONFIG_HOME, VIA_OPENCLAW_MO…` | shared/backend-catalog.mjs:33-34,42,70-73,387 — per-backend environment.names and workspaceEnvironment / expl… |
| `QWEN_AUDIO_AGENT_PERSONAL_OWNER_ID` | `VIA_PERSONAL_OWNER_ID` | shared/runtime-environment.mjs:533 (default value 'user_personal') |
| `qwen_audio_agent_session_cancel` | `via_session_cancel` | server/src/agent/acp-session-tools.mjs:15,114 — MCP tool name; interpolated into the default cancelInstructio… |
| `qwen_audio_agent_session_send` | `via_session_send` | server/src/agent/acp-session-tools.mjs:13,74 — MCP tool name |
| `qwen_audio_agent_session_start` | `via_session_start` | server/src/agent/acp-session-tools.mjs:12,56 — MCP tool name; referenced in the default sessionInstructions p… |
| `qwen_audio_agent_session_status` | `via_session_status` | server/src/agent/acp-session-tools.mjs:14,92 — MCP tool name; also interpolated into the default statusInstru… |
| `qwen_audio_agent_sessions_list` | `via_sessions_list` | server/src/agent/acp-session-tools.mjs:11,34 — MCP tool name; also matched by PermissionBroker's internal-too… |
| `qwen_audio_agent_sessions_list, qwen_audio_agent_session_start, qwen_audio_agent_session_…` | `via_sessions_list, via_session_start, via_session_send, via_session_status, via…` | server/src/agent/acp-session-tools.mjs:10-16,34,56,74,92,114 (registration) and ACP_SESSION_TOOL_NAMES consum… |
| `QWEN_AUDIO_AGENT_SKILLS_CLI_PACKAGE` | `VIA_SKILLS_CLI_PACKAGE` | shared/skill-library.mjs:21 |
| `QWEN_AUDIO_ALLOW_UNCONFIGURED` | `VIA_ALLOW_UNCONFIGURED` | shared/gateway-setup.mjs:38,41 |
| `QWEN_AUDIO_GATEWAY_INSTANCE_ID, QWEN_AUDIO_GATEWAY_STARTED_AT` | `VIA_GATEWAY_INSTANCE_ID, VIA_GATEWAY_STARTED_AT` | server/src/index.mjs:55-56, echoed by /api/health at gateway-application.mjs:237-238; consumed indirectly by … |
| `QWEN_AUDIO_GATEWAY_OWNER` | `VIA_GATEWAY_OWNER` | shared/gateway-options.mjs:37; server/src/index.mjs:52 |
| `QWEN_AUDIO_GATEWAY_OWNER / QWEN_AUDIO_LOG_CONSOLE / QWEN_AUDIO_LOG_DIR / QWEN_AUDIO_LOG_L…` | `VIA_GATEWAY_OWNER, VIA_LOG_CONSOLE, VIA_LOG_DIR, VIA_LOG_LEVEL, VIA_LOG_FILE, V…` | cli/src/gateway-service.mjs:64,65; shared/logger.mjs:51,248,256-272; shared/gateway-options.mjs:37,39 |
| `QWEN_AUDIO_LOG_LEVEL, QWEN_AUDIO_LOG_DIR, QWEN_AUDIO_LOG_CONSOLE, QWEN_AUDIO_LOG_FILE, QW…` | `VIA_LOG_LEVEL, VIA_LOG_DIR, VIA_LOG_CONSOLE, VIA_LOG_FILE, VIA_LOG_MAX_BYTES, V…` | shared/logger.mjs:51,248,256,257,266,272; shared/gateway-options.mjs:39; shared/runtime-environment.mjs:43-45… |
| `QWEN_AUDIO_MEMORY_AUTO / QWEN_AUDIO_MEMORY_MODEL / QWEN_AUDIO_MEMORY_BASE_URL / QWEN_AUDI…` | `VIA_MEMORY_AUTO / VIA_MEMORY_MODEL / VIA_MEMORY_BASE_URL / VIA_MEMORY_API_KEY` | server/src/core/config.mjs:447-458 — the memory extractor's model/endpoint/key settings consumed by server/sr… |
| `QWEN_AUDIO_REALTIME_MODEL / QWEN_AUDIO_REALTIME_VOICE / QWEN_AUDIO_REALTIME_API_KEY / QWE…` | `VIA_REALTIME_MODEL, VIA_REALTIME_VOICE (audio family), VIA_OMNI_REALTIME_VOICE …` | cli/src/config-command.mjs:24,25,51,60; cli/src/launcher.mjs:183,207; cli/src/runtime.mjs:169; shared/realtim… |
| `QWEN_AUDIO_REALTIME_PROVIDER, QWEN_AUDIO_REALTIME_API_KEY, QWEN_AUDIO_REALTIME_BASE_URL, …` | `VIA_REALTIME_PROVIDER, VIA_REALTIME_API_KEY, VIA_REALTIME_BASE_URL, VIA_REALTIM…` | shared/realtime-provider-catalog.mjs:56,85,87,90,91,99; shared/runtime-environment.mjs:24,600; .env.example |
| `QWEN_AUDIO_REALTIME_PROVIDER, QWEN_AUDIO_REALTIME_API_KEY, QWEN_AUDIO_REALTIME_BASE_URL, …` | `VIA_REALTIME_PROVIDER, VIA_REALTIME_API_KEY, VIA_REALTIME_BASE_URL, VIA_REALTIM…` | shared/realtime-provider-catalog.mjs:56-58,85-104 — env vars read by resolveRealtimeFrontendConfiguration. |
| `qwen_audio_request_id` | `via_request_id` | server/src/voice/providers/ga-protocol.mjs:25 — RESPONSE_CORRELATION_KEY; written into response.create as res… |
| `QWEN_AUDIO_WAKE_WORD_ENABLED` | `VIA_WAKE_WORD_ENABLED` | shared/gateway-options.mjs:35 |
| `QWEN_AUDIO_WAKE_WORD_ENABLED, QWEN_AUDIO_WAKE_WORD_MODEL_DIR, QWEN_AUDIO_DESKTOP_AUTO_HID…` | `VIA_WAKE_WORD_ENABLED, VIA_WAKE_WORD_MODEL_DIR, VIA_SLEEP_TIMEOUT_SECONDS` | server/src/core/config.mjs:500-506 — env vars driving the wake-word pipeline and sleep timeout in this scope |
| `QWEN_AUDIO_WAKE_WORD_ENABLED, QWEN_AUDIO_WAKE_WORD_MODEL_DIR, QWEN_AUDIO_DESKTOP_AUTO_HID…` | `VIA_WAKE_WORD_ENABLED, VIA_WAKE_WORD_MODEL_DIR, VIA_DESKTOP_AUTO_HIDE_SECONDS, …` | server/src/core/config.mjs:196-204,333-363,402-406,495-506; docs/contract.md:165 — env vars that reach the re… |
| `QWEN_OMNI_REALTIME_VOICE` | `VIA_OMNI_REALTIME_VOICE` | shared/realtime-provider-catalog.mjs:57 |
| `qwenaudio` | `via` | package.json:50 bin name; README/CHANGELOG; test/verify-package.test.mjs, test/consumer-install.test.mjs |
| `qwenaudio (CLI bin name; also in the user-facing error '缺少 DASHSCOPE_API_KEY。请运行 qwenaudi…` | `via (binary), and the message becomes '缺少 DASHSCOPE_API_KEY。请运行 via config 查看配置…` | package.json:50; shared/realtime-provider-catalog.mjs:152; shared/runtime-environment.mjs:608 |
| `qwenaudio (CLI binary name inside user-facing strings)` | `via` | shared/realtime-provider-catalog.mjs:152; shared/runtime-environment.mjs:608; shared/skill-library.mjs:10 (co… |
| `qwenaudio (CLI binary) / qwen-audio-agent (package name and import specifiers)` | `via (binary), via (crate/package name), via::realtime_events / via::realtime_pr…` | cli/bin/qwenaudio.mjs; docs/contract.md:56-72 package entry points (`qwen-audio-agent/realtime-events`, `qwen… |
| `Symbol('qwen-audio-agent.builtin-mcp-lifecycle')` | `via.builtin-mcp-lifecycle` | server/src/agent/builtin-mcp.mjs:17 |
| `The qwen_audio_agent MCP tools are the only interface for opening, continuing, querying, …` | `The via MCP tools are the only interface for opening, continuing, querying, and…` | server/src/agent/acp-backend-adapter.mjs:942 — default profile.sessionInstructions |
| `tmp staging prefix `qwen-audio-agent-install-` and test dir prefixes `qwenaudio-cli-lock-…` | `via-install-, via-cli-lock-, via-config-` | scripts/install-global.mjs:14; cli/test/instance-lock.test.mjs:9; cli/test/launcher.test.mjs (several) |
| `You are the backend Agent for qwen-audio-agent. / Treat the qwen-audio-agent request enve…` | `You are the backend Agent for VIA. / Treat the VIA request envelope as the curr…` | server/src/agent/backend-agent-instructions.mjs:2,5,14 — BACKEND_AGENT_INSTRUCTIONS, injected into every coor… |
| `~/.config/qwaudio` | `~/.config/via` | shared/runtime-environment.mjs:97-101; scripts/openclaw.mjs:25,27; shared/logger.mjs:56; docs/getting-started… |
| `~/.config/qwaudio  (resolve(XDG_CONFIG_HOME \|\| ~/.config, 'qwaudio'))` | `~/.config/via` | shared/runtime-environment.mjs:97-100 — the default directory holding every file my scope persists |
| `~/.config/qwaudio and ~/.config/qwaudio/logs` | `~/.config/via and ~/.config/via/logs` | shared/runtime-environment.mjs:100; shared/logger.mjs:56 |
| `你好千问` | `VIA needs its own Chinese wake word — it MUST be a 3-5 syllable phrase that the…` | server/src/core/config.mjs:503 (config.wakeWord); server/src/voice/wake-word/model-manager.mjs:95 (sherpa-onn… |
| `你好千问 (config.wakeWord)` | `a VIA-branded wake phrase (e.g. 你好 VIA), CONDITIONAL on shipping a matching she…` | server/src/core/config.mjs:503; sent to clients in every voice.sleep event and in the '已休眠，请先说“…”唤醒。' error, … |
| `你好千问 (wake phrase) and its pinyin token line 'n ǐ h ǎo q iān w èn @你好千问'` | `a VIA wake phrase, e.g. 你好 VIA / 'n ǐ h ǎo v i a @你好VIA' — the exact tokenizati…` | server/src/core/config.mjs:503 (config.wakeWord) and server/src/voice/wake-word/model-manager.mjs:95 (generat… |
| `千问 Audio` | `VIA (docs); the desktop occurrence is out of scope (Electron) but must be renam…` | docs/reference/memory.zh.md:48 (documentation of the ASSISTANT.md scope rule), desktop/src/main.mjs:313 notif… |
| `千问Audio` | `VIA — rewrite the sentence as `没有当前用户的个性化覆盖时，你叫 VIA。` (add the space before the…` | config/frontend-agent/ASSISTANT.md:3 — model-visible persona identity, inside `没有当前用户的个性化覆盖时，你叫千问Audio。`. Pin… |
| `请先配置 DASHSCOPE_API_KEY / 缺少 DASHSCOPE_API_KEY。请运行 qwenaudio config 查看配置文件位置。` | `缺少 DASHSCOPE_API_KEY。请运行 via config 查看配置文件位置。  (keep DASHSCOPE_API_KEY, rename …` | server/src/voice/providers/dashscope.mjs:63; shared/realtime-provider-catalog.mjs:152 — missingConfigurationM… |
| `请调用 qwen_audio_agent_session_cancel 取消 delegation_id=${record.id}。` | `请调用 via_session_cancel 取消 delegation_id=${record.id}。` | server/src/agent/acp-backend-adapter.mjs:1389 — default cancel instruction |
| `请调用 qwen_audio_agent_session_status 查询 delegation_id=${record.id}。` | `请调用 via_session_status 查询 delegation_id=${record.id}。` | server/src/agent/acp-backend-adapter.mjs:1434 — default status instruction |

## KEEP — 65

| String | Why it stays |
| --- | --- |
| `'dashscope' (provider key) and 'DashScope' (label)` | Names the upstream vendor service, not our product; appears in /api/health and in client provider selection. |
| `'Qwen Audio Realtime' (prose)` | 'Qwen Audio Realtime' here names the DashScope model family, which is a KEEP; but the sentence asserts it is *the default frontend*, which is a VIA product statement and should be reframed … |
| `'qwen' (backend agent id) and 'Qwen Code' (label)` | Identifies the third-party Qwen Code CLI backend, not our product. Users set AGENT_PROTOCOL=qwen to select it. |
| `'qwen' (realtime provider alias for dashscope) and REALTIME_PROVIDERS keys 'qwe…` | Explicitly excluded by the rebrand rule: this is the frontend voice engine's qwen runtime name. Clients may still send provider:'qwen' on the WS connect event. |
| `'qwen' as a backend protocol id (Qwen Code CLI) and QWEN_CODE_BIN / QWEN_CODE_W…` | Names a third-party product (Qwen Code) VIA integrates with. Its env vars belong to that product's namespace. |
| `'qwen' backend workspace id / qwenCodeWorkspace` | Refers to the Qwen Code backend agent product, i.e. a third-party tool name rather than this product's identity. Flagged here for completeness; it belongs to the backend-catalog survey. |
| `'Qwen-Audio-Realtime' (dashscope provider label) and aliases ['qwen']` | Explicit exemption (b) in the rebrand rule: the frontend voice engine's qwen runtime keeps its name. Clients select this provider by the 'qwen' alias. |
| `'qwen-code' (skills.sh installer id) and '.qwen/skills'` | A skills.sh agent id and the Qwen Code CLI's own global skills directory - both third-party conventions. |
| `'qwen-flash' (default QWEN_AUDIO_MEMORY_MODEL)` | A real DashScope model id used by the memory extractor's OpenAI-compatible call. The env var name around it renames to VIA_MEMORY_MODEL; the default value does not. |
| `'Qwen3.5 Omni Flash Realtime', 'Qwen3.5 Omni Plus Realtime', 'Qwen Audio 3.0 Re…` | Human-readable names OF the models, shown in a model picker. They name the vendor's product, not ours. |
| `'realtime-direct' / 'agent-presentation' / 'voice-user' / 'text-user' (conversa…` | No qwen branding; listed here because they are persisted enum values that the port must reproduce byte-for-byte. |
| `'the alternate Qwen ASR streaming event name'` | A provider wire event name that must be recognized as-is; it is inbound data, not our identity. |
| `'voice-' turn id prefix and 'text_' turn id prefix; notificationClaimantId 'voi…` | Generic, brand-free identifiers. Listed only to confirm they carry no qwen identity and need no change. |
| `@qwen-code/open-computer-use` | Third-party npm package name. The Rust port either vendors an equivalent or keeps resolving this exact specifier. |
| `@qwen-code/open-computer-use (package) / 'open-computer-use' (bin + MCP descrip…` | Upstream npm package published under the qwen-code scope; VIA cannot rename someone else's package, and the MCP descriptor name is the tool namespace the model sees for a third-party tool s… |
| `@qwen-code/qwen-code@0.21.6` | Third-party npm package coordinate. |
| `acp-sessions.json` | Not a qwen-branded name; it describes the ACP protocol content. Keeping it also lets the Rust port read an existing install's state file unchanged. |
| `Agent 结果` | Generic Chinese UI label ('Agent result'), carries no qwen identity. Listed so the port does not accidentally 'rebrand' it. |
| `aliases: ['qwen'] / REALTIME_PROVIDERS.qwen` | This alias names the Qwen cloud realtime service (the model vendor), not the product. Users type `provider=qwen` meaning 'the Qwen realtime API'. Renaming it would break existing configs fo… |
| `All GatewayClientEvent / GatewayServerEvent / GatewayTaskEvent names and every …` | No route or event name carries the qwen brand ('/api/health', '/api/realtime', 'voice.state', 'task.completed', …). They are pure protocol and must survive the rebrand byte for byte. |
| `backend id 'qwen' plus QWEN_CODE_BIN, QWEN_CODE_WORKSPACE, QWEN_CODE_PACKAGE, Q…` | These name the third-party Qwen Code CLI and its own environment contract. Renaming them would break integration with a product this repo does not own. |
| `backend id `qwen` / label `Qwen Code` and its env names QWEN_CODE_BIN, QWEN_COD…` | Third-party product (Alibaba's Qwen Code CLI) — its id, executable, npm package and config directory are owned by that project. |
| `backend model names 'qwen3.7-max', 'qwen3.7-plus', 'qwen-plus', 'qwen3-coder-pl…` | Model registry identifiers resolved by third-party backends. |
| `connectTimeoutMessage '连接 Qwen Audio Realtime 超时' and the dashscope provider ke…` | Names the DashScope 'Qwen Audio Realtime' service endpoint the frontend voice engine connects to — the explicitly excepted qwen runtime. Out of this survey's file scope; flagged so it is no… |
| `coordinator:<ownerId> (laneKey), user_preferences / user_memory / assistant_pro…` | No product or model name; the prompt tag names are model-facing contract text that PROMPT.md refers to by name, so changing them would silently break the instruction hierarchy. |
| `DASHSCOPE_API_KEY, dashscope provider key, dashscope.aliyuncs.com base URLs` | Vendor credential and vendor service identity. Renaming would be actively wrong. |
| `DASHSCOPE_API_KEY, DASHSCOPE_WORKSPACE_ID, `bailian`, dashscope.aliyuncs.com UR…` | Alibaba Cloud service identifiers and endpoints. Not qwen-branded and not ours to rename. |
| `DASHSCOPE_API_KEY, DASHSCOPE_WORKSPACE_ID, dashscope.aliyuncs.com` | Alibaba DashScope's own credential variable and endpoints, and the conventional name several third-party backends read. Renaming breaks both the upstream API and every backend's Bailian aut… |
| `DASHSCOPE_API_KEY, DASHSCOPE_WORKSPACE_ID, dashscope.aliyuncs.com base URLs` | Alibaba Cloud DashScope is the cloud model provider; these are the vendor's own credential and endpoint names, not VIA branding. They stay even though the port must additionally support on-… |
| `DASHSCOPE_API_KEY, DASHSCOPE_WORKSPACE_ID, dashscope.aliyuncs.com endpoints, th…` | Third-party service credentials, hosts and provider prefixes owned by Alibaba Cloud. |
| `DASHSCOPE_API_KEY, DASHSCOPE_WORKSPACE_ID, SPEECH_TO_SPEECH_REALTIME_URL, S2S_R…` | These name external services (Alibaba DashScope, huggingface/speech-to-speech), not our product. Users already have DASHSCOPE_API_KEY set from the vendor's own docs. |
| `gateway.log, gateway.lock, config.env, state.env, USER.md, ASSISTANT.md, MEMORY…` | Brand-free file names inside the (renamed) config directory. Keeping them makes the ~/.config/qwaudio -> ~/.config/via migration a pure directory move. |
| `GATEWAY_CAPABILITIES member strings ('web.same-origin-ui', 'gateway.instance-le…` | None of the capability VALUES contain the qwen name — only the surrounding comments do (they reference qwen-audio-agent/electron, /gateway-process, /orb/preload, /skin-store subpaths). The … |
| `https://dashscope.aliyuncs.com/compatible-mode/v1` | Third-party vendor endpoint, not a qwen identity string. For the on-device port it should become overridable to a local OpenAI-compatible endpoint (the code already documents Ollama), but t… |
| `initialize, session/new, session/load, session/resume, session/prompt, session/…` | Agent Client Protocol wire method names owned by an external standard. Renaming any of them breaks every backend agent. |
| `longanqian / Ethan` | DashScope voice ids; the service rejects unknown values. |
| `probe {kind:'qwen-settings'} and skills {installer:'qwen-code'}` | Both name the third-party Qwen Code product: the probe inspects Qwen Code's own settings file and the installer id is the agent id registered with skills.sh upstream (and is validated again… |
| `provider alias key `qwen`` | Explicitly carved out by the rebrand rule: this is the frontend voice engine's qwen runtime. It is also a back-compat alias that users may already have in QWEN_AUDIO_REALTIME_PROVIDER=qwen,… |
| `qoder / qodercli / Qoder` | Third-party CLI, not a qwen-family identity at all; listed for completeness because it sits in the same table as the `qwen` driver. |
| `QODER_AUTH_MODE` | Not a qwen string, but flagged here because it is DEAD: documented in .env.example and never read by any code. Do not port it; delete the line. |
| `qwen` | This is the third-party Qwen Code CLI, an external product. The driver id doubles as the coordinator-key prefix and the env-namespace key, but it names the backend, not VIA. Renaming it wou… |
| `qwen (backend id) / 'Qwen Code' (label) / 'qwen' (command) / args ['--acp']` | Third-party product identity: the Qwen Code CLI published by Alibaba. The id is a protocol selector matched against the vendor's own name, the label is validated to equal the catalog label,… |
| `qwen (realtime provider alias for dashscope)` | This is the frontend voice engine's qwen runtime alias, explicitly exempt from the rebrand. Users set the provider to 'qwen' today and clients may send provider:'qwen' in the connect event. |
| `Qwen Audio 3.0 Realtime Plus / Flash, Qwen3.5 Omni Flash Realtime, Qwen3.5 Omni…` | Human labels for the same third-party models. |
| `qwen-audio-3.0-realtime-plus / qwen-audio-3.0-realtime-flash / qwen3.5-omni-fla…` | Actual DashScope model identifiers. Renaming them would make the API call fail. |
| `qwen-audio-3.0-realtime-plus, qwen-audio-3.0-realtime-flash, qwen3.5-omni-flash…` | Actual model identifiers sent to DashScope on the wire. Changing them breaks the connection. Their human labels ('Qwen3.5 Omni Flash Realtime', etc.) are also KEEP. |
| `Qwen-Audio-Realtime` | It is the display name of the Qwen model service the provider connects to. Note the sibling label in shared/realtime-provider-catalog.mjs:25 is 'DashScope' — the two disagree today; the por… |
| `qwen-flash` | This is an actual model name, explicitly excluded from the rebrand. Changing it would break the extractor against DashScope. |
| `qwen-flash (memoryModel default), qwen backend model profile key` | Actual model names / model-profile keys, explicitly excepted by the rebrand rule. |
| `Qwen3.5 Omni Flash Realtime / Qwen3.5 Omni Plus Realtime / Qwen Audio 3.0 Realt…` | Human-readable names of the same Qwen models. |
| `qwen3.5-omni-flash-realtime, qwen3.5-omni-plus-realtime, qwen-audio-3.0-realtim…` | Vendor model identifiers sent verbatim to DashScope. Renaming breaks the API call. Explicitly exempted by the rebrand rule. |
| `qwen3.7-max / "Qwen 3.7 Max"` | Actual DashScope model id and its display name, vendor 'Alibaba Cloud Model Studio'. Renaming would make the CodeBuddy backend request a non-existent model. |
| `QWEN_API_KEY, QWEN_OAUTH_TOKEN, QWEN_HOME, ~/.qwen/settings.json` | Qwen Code's own credential variables and config path, read read-only during the auth probe. They belong to the other product. |
| `QWEN_CODE_BIN, QWEN_CODE_WORKSPACE, and the value `qwen` for AGENT_PROTOCOL` | Qwen Code is a separate third-party product with its own CLI (`qwen --acp`), its own home directory (~/.qwen) and its own env-var names. Renaming these would break integration with the user… |
| `QWEN_CODE_BIN, QWEN_CODE_WORKSPACE, QWEN_CODE_PACKAGE, prefix 'QWEN_CODE_'` | Third-party product's env namespace, forwarded to the Qwen Code child process. QWEN_CODE_WORKSPACE and QWEN_CODE_BIN are read by VIA but named after the vendor product they configure; renam… |
| `QWEN_CODE_WORKSPACE, QWEN_CODE_BIN, QWEN_CODE_PACKAGE and the QWEN_CODE_ prefix` | QWEN_CODE_BIN/PACKAGE address the third-party CLI and its npm package, and the QWEN_CODE_ prefix is the allowlist for that vendor's own variables. Renaming breaks users' existing Qwen Code … |
| `realtime provider alias 'qwen' → 'dashscope'` | Explicitly exempted: this is the frontend voice engine's qwen runtime alias, accepted from user config. |
| `sherpa-onnx-kws-zipformer-zh-en-3M-2025-12-20 and https://github.com/k2-fsa/she…` | Third-party upstream model id and release URL from the k2-fsa/sherpa-onnx project. Not a qwen identity string at all, and renaming would break the download and the SHA-256 pin. |
| `speech-to-speech / s2s / 'Hugging Face Speech-to-Speech'` | Names the third-party huggingface/speech-to-speech project. Not qwen-branded and not product identity. |
| `tasks.json / frontend-notes.json / memory-audit.jsonl / state/acp-sessions.json` | Generic filenames carrying no product or model identity. |
| `USER.md / MEMORY.md / ASSISTANT.md / PROMPT.md` | No qwen name; these filenames are also embedded verbatim in EXTRACTOR_SYSTEM_PROMPT and in PROMPT.md's instruction hierarchy, so renaming them would require rewriting model-facing prompt te… |
| `voice ids 'Ethan' and 'longanqian'` | DashScope-owned voice identifiers sent in session.update. |
| `work_ (Work id prefix) / item_ (note item id prefix) / agent: (conversation mes…` | Generic identifiers with no qwen name. They ARE external contracts (the model round-trips work_ ids and item_ ids), so they must be preserved byte-for-byte regardless. |
| `仅前台聊天` | Generic Chinese label ('frontend chat only') describing the voice-only deployment mode; no qwen identity. |
| `连接 Qwen Audio Realtime 超时` | Names the remote Qwen service that timed out, which is the useful information. Only the surrounding product chrome would be renamed, and there is none here. |
