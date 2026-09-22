use gate2_contracts::WorkerRequest;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines},
    net::{TcpListener, TcpStream},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::{Instant, sleep, sleep_until, timeout},
};

#[derive(Clone, Copy)]
struct Profile {
    name: &'static str,
    external_repeats: usize,
    fault_hold: Duration,
    probe_offsets: [Duration; 3],
    capability_deadline: Duration,
    fatal_repeats: usize,
    fatal_deadline: Duration,
    metric_eligible: bool,
}

impl Profile {
    fn from_name(name: &str) -> anyhow::Result<Self> {
        match name {
            "smoke" => Ok(Self {
                name: "smoke",
                external_repeats: 1,
                fault_hold: Duration::from_millis(50),
                probe_offsets: [
                    Duration::from_millis(2),
                    Duration::from_millis(10),
                    Duration::from_millis(20),
                ],
                capability_deadline: Duration::from_millis(5),
                fatal_repeats: 1,
                fatal_deadline: Duration::from_millis(2000),
                metric_eligible: false,
            }),
            "frozen" => Ok(Self {
                name: "frozen",
                external_repeats: 5,
                fault_hold: Duration::from_secs(30),
                probe_offsets: [
                    Duration::from_secs(2),
                    Duration::from_secs(10),
                    Duration::from_secs(20),
                ],
                capability_deadline: Duration::from_secs(5),
                fatal_repeats: 5,
                fatal_deadline: Duration::from_secs(5),
                metric_eligible: false,
            }),
            other => anyhow::bail!("unknown W-10 profile: {other}; use smoke or frozen"),
        }
    }
}

fn expected_capabilities(spec: &Value) -> anyhow::Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for capability in spec["capabilities"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("W-10 capabilities must be an array"))?
    {
        let id = capability["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("W-10 capability id missing"))?;
        let token = capability["expected_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("W-10 capability expected_token missing"))?;
        anyhow::ensure!(
            out.insert(id.to_string(), token.to_string()).is_none(),
            "duplicate W-10 capability {id}"
        );
    }
    Ok(out)
}

fn validate_spec(spec: &Value) -> anyhow::Result<()> {
    anyhow::ensure!(spec["version"] == "W10-G2-v1", "unexpected W-10 spec version");
    anyhow::ensure!(
        spec["metric"]["formula"] == "100 * passed_cells / 28",
        "W-10 denominator/formula changed"
    );
    anyhow::ensure!(
        spec["metric"]["cell_pass_rule"] == "STRICT_ALL_TRIALS_PASS",
        "W-10 cell rule must remain strict"
    );

    let external = spec["external_cells"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("external_cells missing"))?;
    let fatal = spec["integration_fatal_cells"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("integration_fatal_cells missing"))?;
    anyhow::ensure!(external.len() == 24, "W-10 requires exactly 24 external cells");
    anyhow::ensure!(fatal.len() == 4, "W-10 requires exactly 4 fatal cells");

    let frozen = &spec["frozen_profile"];
    anyhow::ensure!(
        frozen["external"]["failure_modes"]
            == json!(["connection_refused", "no_reply"]),
        "W-10 external failure modes changed"
    );
    anyhow::ensure!(
        frozen["external"]["repeats_per_mode"] == 5,
        "W-10 external repeat count changed"
    );
    anyhow::ensure!(
        frozen["external"]["fault_hold_ms"] == 30000,
        "W-10 fault hold changed"
    );
    anyhow::ensure!(
        frozen["external"]["probe_offsets_ms"] == json!([2000, 10000, 20000]),
        "W-10 probe offsets changed"
    );
    anyhow::ensure!(
        frozen["external"]["capability_deadline_ms"] == 5000,
        "W-10 capability deadline changed"
    );
    anyhow::ensure!(
        frozen["integration_fatal"]["repeats"] == 5,
        "W-10 fatal repeat count changed"
    );

    let capabilities = spec["capabilities"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("capabilities missing"))?;
    let mut cap_dependencies: HashMap<&str, HashSet<&str>> = HashMap::new();
    for capability in capabilities {
        let id = capability["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("capability id missing"))?;
        let deps = capability["intrinsic_dependencies"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("intrinsic_dependencies missing for {id}"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("non-string dependency for {id}"))
            })
            .collect::<anyhow::Result<HashSet<_>>>()?;
        cap_dependencies.insert(id, deps);
    }

    let dependencies = spec["dependencies"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("dependencies missing"))?;
    anyhow::ensure!(dependencies.len() == 6, "W-10 requires exactly six external dependencies");
    let dependency_ids = dependencies
        .iter()
        .map(|dependency| {
            dependency["id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("dependency id missing"))
        })
        .collect::<anyhow::Result<HashSet<_>>>()?;
    anyhow::ensure!(dependency_ids.len() == 6, "duplicate W-10 dependency");

    let mut ids = HashSet::new();
    let mut dependency_counts: HashMap<&str, usize> = HashMap::new();
    for cell in external {
        let id = cell["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("external cell id missing"))?;
        anyhow::ensure!(ids.insert(id), "duplicate W-10 cell id {id}");
        let dependency = cell["failed_dependency"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("failed_dependency missing in {id}"))?;
        let capability = cell["unaffected_capability"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unaffected_capability missing in {id}"))?;
        anyhow::ensure!(
            dependency_ids.contains(dependency),
            "unknown W-10 dependency {dependency}"
        );
        let intrinsic = cap_dependencies
            .get(capability)
            .ok_or_else(|| anyhow::anyhow!("unknown W-10 capability {capability}"))?;
        anyhow::ensure!(
            !intrinsic.contains(dependency),
            "{id} is invalid: {capability} intrinsically depends on failed {dependency}"
        );
        *dependency_counts.entry(dependency).or_default() += 1;
    }
    for dependency in dependency_ids {
        anyhow::ensure!(
            dependency_counts.get(dependency) == Some(&4),
            "W-10 dependency {dependency} must contribute exactly four cells"
        );
    }

    for cell in fatal {
        let id = cell["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("fatal cell id missing"))?;
        anyhow::ensure!(ids.insert(id), "duplicate W-10 cell id {id}");
        anyhow::ensure!(
            cell["fault"] == "integration_host_fatal",
            "{id} must use the frozen integration_host_fatal fault"
        );
        let capability = cell["unaffected_capability"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("fatal capability missing in {id}"))?;
        anyhow::ensure!(
            cap_dependencies.contains_key(capability),
            "unknown fatal capability {capability}"
        );
    }
    anyhow::ensure!(ids.len() == 28, "W-10 requires exactly 28 unique cells");
    Ok(())
}

async fn fixture_capability(capability: &str) -> anyhow::Result<&'static str> {
    Ok(match capability {
        "CAP-TEXT-INTERACTION" => "text:ready",
        "CAP-S2S-DIRECT" => "s2s:direct-ready",
        "CAP-SEMANTIC-TEXT" => "semantic:ready",
        "CAP-BOUNDED-DOC-READ" => "doc-budget:education-budget",
        "CAP-BOUNDED-MAIL-READ" => "mail:education-schedule",
        "CAP-AGENT-DOC-TASK" => "agent-doc:T-PPT:running",
        "CAP-AGENT-MAIL-TASK" => "agent-mail:T-MAIL:running",
        "CAP-LOCAL-MEMORY-READ" => "memory:table-first",
        "CAP-LOCAL-TASK-CARD-READ" => "task:T-PPT:running",
        other => anyhow::bail!("no deterministic W-10 fixture for {other}"),
    })
}

async fn run_external_trial(
    mode: &str,
    capability: &str,
    expected: &str,
    profile: Profile,
) -> anyhow::Result<bool> {
    let mut no_reply: Option<(TcpStream, tokio::task::JoinHandle<()>)> = None;
    let fault_observed = match mode {
        "connection_refused" => {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let address = listener.local_addr()?;
            drop(listener);
            TcpStream::connect(address).await.is_err()
        }
        "no_reply" => {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let address = listener.local_addr()?;
            let hold = profile.fault_hold;
            let server = tokio::spawn(async move {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut request = [0_u8; 16];
                    let _ = socket.read(&mut request).await;
                    sleep(hold).await;
                }
            });
            let mut stream = TcpStream::connect(address).await?;
            stream.write_all(b"w10").await?;
            no_reply = Some((stream, server));
            true
        }
        other => anyhow::bail!("unknown external W-10 fault mode {other}"),
    };

    let start = Instant::now();
    let mut probes_ok = fault_observed;
    for offset in profile.probe_offsets {
        sleep_until(start + offset).await;
        let observed = timeout(profile.capability_deadline, fixture_capability(capability)).await;
        probes_ok &= matches!(observed, Ok(Ok(value)) if value == expected);
    }

    if let Some((mut stream, server)) = no_reply {
        let mut byte = [0_u8; 1];
        let still_no_reply = timeout(profile.capability_deadline, stream.read(&mut byte))
            .await
            .is_err();
        probes_ok &= still_no_reply;
        server.abort();
    }
    Ok(probes_ok)
}

struct HostSession {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl HostSession {
    async fn spawn(host: &Path, mode: &str, worker: Option<&Path>) -> anyhow::Result<Self> {
        let mut command = Command::new(host);
        command
            .arg("--mode")
            .arg(mode)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true);
        if let Some(worker) = worker {
            command.arg("--worker").arg(worker);
        }
        let mut child = command.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("W-10 host stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("W-10 host stdout missing"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }

    async fn request(&mut self, value: Value) -> anyhow::Result<Option<Value>> {
        self.stdin
            .write_all(serde_json::to_string(&value)?.as_bytes())
            .await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        match self.stdout.next_line().await? {
            Some(line) => Ok(Some(serde_json::from_str(&line)?)),
            None => Ok(None),
        }
    }
}

async fn integration_trial(
    host: &Path,
    worker: &Path,
    mode: &str,
    capability: &str,
    expected: &str,
    deadline: Duration,
) -> anyhow::Result<bool> {
    let isolated = mode == "isolated";
    let mut session = HostSession::spawn(host, mode, isolated.then_some(worker)).await?;
    let ping = timeout(deadline, session.request(json!({"op":"ping"}))).await??;
    anyhow::ensure!(
        matches!(ping, Some(value) if value["status"] == "pong"),
        "W-10 candidate host did not start"
    );

    let abort = timeout(
        deadline,
        session.request(serde_json::to_value(WorkerRequest::AbortHost)?),
    )
    .await;

    if !isolated {
        let reply = abort??;
        anyhow::ensure!(
            reply.is_none(),
            "shared-process fatal fault unexpectedly returned normally"
        );
        let _ = session.child.wait().await?;
        return Ok(false);
    }

    let reply = abort??;
    anyhow::ensure!(
        matches!(reply, Some(value) if value["status"] == "pong"),
        "isolated Core host did not survive worker fatal fault"
    );
    anyhow::ensure!(
        session.child.try_wait()?.is_none(),
        "isolated Core process exited after integration worker fatal fault"
    );

    let probe = timeout(
        deadline,
        session.request(json!({"op":"core_probe","capability":capability})),
    )
    .await??;
    Ok(matches!(
        probe,
        Some(value)
            if value["status"] == "core_probe"
                && value["capability"] == capability
                && value["value"] == expected
    ))
}

pub async fn run(
    spec_path: PathBuf,
    host: PathBuf,
    worker: PathBuf,
    profile_name: String,
) -> anyhow::Result<Value> {
    let spec: Value = serde_json::from_slice(&std::fs::read(&spec_path)?)?;
    validate_spec(&spec)?;
    let expected = expected_capabilities(&spec)?;
    let profile = Profile::from_name(&profile_name)?;

    let mut external_results = Vec::new();
    for cell in spec["external_cells"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("external_cells missing"))?
    {
        let cell_id = cell["id"].as_str().unwrap();
        let capability = cell["unaffected_capability"].as_str().unwrap();
        let expected_token = expected
            .get(capability)
            .ok_or_else(|| anyhow::anyhow!("missing expected token for {capability}"))?;
        let mut trials = Vec::new();
        for mode in ["connection_refused", "no_reply"] {
            for repeat in 0..profile.external_repeats {
                let pass = run_external_trial(mode, capability, expected_token, profile).await?;
                trials.push(json!({
                    "mode": mode,
                    "repeat": repeat + 1,
                    "pass": pass
                }));
            }
        }
        let pass = trials.iter().all(|trial| trial["pass"] == true);
        external_results.push(json!({
            "cell_id": cell_id,
            "failed_dependency": cell["failed_dependency"],
            "unaffected_capability": capability,
            "pass": pass,
            "trials": trials
        }));
    }

    let external_passed = external_results
        .iter()
        .filter(|cell| cell["pass"] == true)
        .count();

    let mut candidate_results = Vec::new();
    for mode in ["shared", "isolated"] {
        let mut fatal_results = Vec::new();
        for cell in spec["integration_fatal_cells"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("integration_fatal_cells missing"))?
        {
            let cell_id = cell["id"].as_str().unwrap();
            let capability = cell["unaffected_capability"].as_str().unwrap();
            let expected_token = expected
                .get(capability)
                .ok_or_else(|| anyhow::anyhow!("missing expected token for {capability}"))?;
            let mut trials = Vec::new();
            for repeat in 0..profile.fatal_repeats {
                let pass = integration_trial(
                    &host,
                    &worker,
                    mode,
                    capability,
                    expected_token,
                    profile.fatal_deadline,
                )
                .await?;
                trials.push(json!({"repeat":repeat + 1,"pass":pass}));
            }
            let pass = trials.iter().all(|trial| trial["pass"] == true);
            fatal_results.push(json!({
                "cell_id":cell_id,
                "unaffected_capability":capability,
                "pass":pass,
                "trials":trials
            }));
        }
        let fatal_passed = fatal_results
            .iter()
            .filter(|cell| cell["pass"] == true)
            .count();
        let passed_cells = external_passed + fatal_passed;
        candidate_results.push(json!({
            "exec_candidate":mode,
            "external_cells_passed":external_passed,
            "integration_fatal_cells_passed":fatal_passed,
            "passed_cells":passed_cells,
            "structural_smoke_retention_pct":100.0 * passed_cells as f64 / 28.0,
            "fatal_cells":fatal_results
        }));
    }

    Ok(json!({
        "status":"PASS",
        "scope":"W-10 28-cell containment controller structural smoke",
        "spec":spec_path,
        "spec_version":spec["version"],
        "profile":profile.name,
        "frozen_contract_validated":true,
        "external_cells":external_results,
        "candidates":candidate_results,
        "w10_representative_metric":"NOT_RUN",
        "w10_metric_eligible":profile.metric_eligible,
        "external_capability_evidence":"DETERMINISTIC_FIXTURE",
        "integration_fatal_evidence":"ACTUAL_SHARED_VS_ISOLATED_PROCESS_BOUNDARY",
        "note":"Smoke validates the frozen 28-cell controller shape and process-fatal containment. Final W-10 requires all frozen repeats/timing and candidate endpoint adapters; smoke percentages are not the representative metric."
    }))
}
