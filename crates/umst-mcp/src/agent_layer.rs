// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! MCP Physical Reasoning Layer — **pure session threading** at the impure stdio boundary.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use umst_cli::cli::{predict_with_options, serialize_prediction, PredictOptions};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::facade::{MixSpec, PredictionWireVersion};
use umst_concrete_cartridge::research::{
    accept, append_gate_reject_jsonl, append_memory_jsonl, estimate_mi_bits_from_mix,
    gate_check_mix_result, mix_wire_from_spec_value, query_page, synthetic_observed_at,
    AcceptError, AcceptResult, GateCheckResult, GateContext, MemoryQuery, MemoryQueryPage,
    ProvenanceClock, ResearchStore, StoreError, WallClock, CANON_VERSION, CONTRIBUTION_SCHEMA,
    DEFAULT_CATALOG_HASH,
};

const JSON_SCHEMA_2020: &str = "https://json-schema.org/draft/2020-12/schema";

fn contribute_jobs_path() -> Option<PathBuf> {
    std::env::var("UMST_MEMORY_DB").ok().map(|db| {
        let parent = Path::new(&db).parent().unwrap_or_else(|| Path::new("."));
        parent.join("contribute_jobs.json")
    })
}

fn load_contribute_jobs() -> HashMap<String, ContributeJob> {
    if let Ok(store) = ResearchStore::from_env() {
        if let Some(raw) = store.load_contribute_jobs_sqlite() {
            let mut jobs = HashMap::new();
            for (id, json) in raw {
                if let Ok(job) = serde_json::from_str::<ContributeJob>(&json) {
                    jobs.insert(id, job);
                }
            }
            if !jobs.is_empty() {
                return jobs;
            }
        }
    }
    let Some(path) = contribute_jobs_path() else {
        return HashMap::new();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn persist_contribute_jobs(jobs: &HashMap<String, ContributeJob>) {
    let json_map: HashMap<String, String> = jobs
        .iter()
        .filter_map(|(id, job)| {
            serde_json::to_string(job)
                .ok()
                .map(|json| (id.clone(), json))
        })
        .collect();
    if let Ok(store) = ResearchStore::from_env() {
        let _ = store.persist_contribute_jobs_sqlite(&json_map);
    }
    let Some(path) = contribute_jobs_path() else {
        return;
    };
    if let Ok(text) = serde_json::to_string_pretty(jobs) {
        let _ = std::fs::write(path, text);
    }
}

fn with_schema_2020(mut tool: Value, read_only: bool) -> Value {
    tool["annotations"] = json!({
        "readOnlyHint": read_only,
        "destructiveHint": false,
    });
    if let Some(schema) = tool.get_mut("inputSchema").and_then(|s| s.as_object_mut()) {
        schema.insert("$schema".into(), json!(JSON_SCHEMA_2020));
    }
    tool
}

/// Async contribute job state (in-memory stub for heavy physics path).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP job status wire; physics on `gate_check_mix` / `accept`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContributeJobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// In-flight async contribute job record.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP async job envelope; gate on synchronous `contribute` delegate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContributeJob {
    pub job_id: String,
    pub status: ContributeJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AcceptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Functional agent session — each mutating tool returns an updated session value.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP session state carrier; thermodynamic gate on cartridge `accept`.
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub store: ResearchStore,
    pub clock: ProvenanceClock,
    pub jobs: HashMap<String, ContributeJob>,
    pub arena_sessions: HashMap<uuid::Uuid, Arc<[u8]>>,
}

impl Default for AgentSession {
    fn default() -> Self {
        Self {
            store: ResearchStore::from_env().unwrap_or_default(),
            clock: ProvenanceClock::default(),
            jobs: load_contribute_jobs(),
            arena_sessions: HashMap::new(),
        }
    }
}

impl AgentSession {
    /// MCP `umst_gate_check` — structured result with optional `gate_reject.v1` + explain.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio JSON-RPC transport; CD admissibility on `gate_check_mix_result`.
    #[must_use]
    pub fn gate_check(&self, profile: &Profile, mix: &Value, explain: bool) -> GateCheckResult {
        let observed = synthetic_observed_at(self.clock.sequence());
        let result = gate_check_mix_result(profile, mix, explain, observed);
        if let Some(ref row) = result.gate_reject {
            let _ = append_gate_reject_jsonl(row, None);
        }
        result
    }

    /// MCP `umst_mi_estimate` — advisory Landauer surrogate only.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: MI enrichment wire; not admissibility gate.
    #[must_use]
    pub fn mi_estimate(&self, mix: &Value) -> Value {
        json!({
            "mi_bits_est": estimate_mi_bits_from_mix(mix),
            "advisory": true,
        })
    }

    /// MCP `umst_contribute` — functional session update via cartridge `accept`.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio transport; gate re-check before memory append on cartridge.
    pub fn contribute(
        self,
        profile: &Profile,
        contribution: &Value,
    ) -> Result<(Self, AcceptResult), AcceptError> {
        let ctx = GateContext { profile };
        match accept(self.store, self.clock, WallClock, &ctx, contribution) {
            Ok((store, clock, result)) => {
                let session = Self {
                    store: store.clone(),
                    clock,
                    jobs: self.jobs,
                    arena_sessions: self.arena_sessions,
                };
                persist_memory_row(&result.memory_id, &store)
                    .map_err(|e| AcceptError::Store(StoreError::Sqlite(e)))?;
                Ok((session, result))
            }
            Err(AcceptError::GateReject(row)) => {
                let _ = append_gate_reject_jsonl(&row, None);
                Err(AcceptError::GateReject(row))
            }
            Err(e) => Err(e),
        }
    }

    /// MCP async contribute stub — returns job id immediately.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: In-process async wrapper; same gate path as `contribute`.
    pub fn contribute_async(self, profile: &Profile, contribution: &Value) -> (Self, String) {
        let job_id = uuid::Uuid::new_v4().to_string();
        let jobs = {
            let mut jobs = self.jobs;
            jobs.insert(
                job_id.clone(),
                ContributeJob {
                    job_id: job_id.clone(),
                    status: ContributeJobStatus::Running,
                    result: None,
                    error: None,
                },
            );
            jobs
        };
        let pending = Self {
            jobs,
            store: self.store,
            clock: self.clock,
            arena_sessions: self.arena_sessions,
        };
        let profile = profile.clone();
        let contribution = contribution.clone();
        match pending.clone().contribute(&profile, &contribution) {
            Ok((session, result)) => {
                let jobs = {
                    let mut jobs = session.jobs;
                    jobs.insert(
                        job_id.clone(),
                        ContributeJob {
                            job_id: job_id.clone(),
                            status: ContributeJobStatus::Succeeded,
                            result: Some(result),
                            error: None,
                        },
                    );
                    persist_contribute_jobs(&jobs);
                    jobs
                };
                (Self { jobs, ..session }, job_id)
            }
            Err(e) => {
                let jobs = {
                    let mut jobs = pending.jobs;
                    jobs.insert(
                        job_id.clone(),
                        ContributeJob {
                            job_id: job_id.clone(),
                            status: ContributeJobStatus::Failed,
                            result: None,
                            error: Some(e.to_string()),
                        },
                    );
                    persist_contribute_jobs(&jobs);
                    jobs
                };
                (
                    Self {
                        jobs,
                        store: pending.store,
                        clock: pending.clock,
                        arena_sessions: pending.arena_sessions,
                    },
                    job_id,
                )
            }
        }
    }

    /// Poll async contribute job status by id.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: In-memory job map lookup; no new physics claim.
    #[must_use]
    pub fn contribute_status(&self, job_id: &str) -> Option<ContributeJob> {
        self.jobs.get(job_id).cloned()
    }

    /// MCP `umst_memory_query` — paginated filter over session store.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio transport over cartridge `query_page`; stable `(ucrs_seq, content_id)` sort.
    #[must_use]
    pub fn memory_query(&self, q: &MemoryQuery) -> MemoryQueryPage {
        query_page(&self.store.rows(), q)
    }

    /// MCP `umst_transition_propose` — predict + gate + async contribute.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Chained operator workflow; gate must pass before async ingest.
    pub fn transition_propose(
        self,
        profile: &Profile,
        mix: &Value,
        outcome: Option<&Value>,
        process: Option<&Value>,
    ) -> Result<(Self, Value), String> {
        let mut spec: MixSpec = mix_wire_from_spec_value(mix)
            .ok_or_else(|| "mix_spec rational parse fail".to_string())
            .and_then(|wire| MixSpec::try_from(wire).map_err(|e| e.to_string()))?;
        spec.profile_name = profile.bundle_id.clone();
        let bundle = predict_with_options(
            profile,
            &spec,
            PredictOptions {
                compare_homogeneous: false,
            },
        )
        .map_err(|e| e.to_string())?;
        let prediction =
            serialize_prediction(&bundle, PredictionWireVersion::V2).map_err(|e| e.to_string())?;

        let gate = self.gate_check(profile, mix, false);
        if !gate.gate_summary.admissible {
            return Err("gate reject: mix not admissible for transition propose".into());
        }

        let observed = synthetic_observed_at(self.clock.sequence());
        let mut outcome_obj = outcome.cloned().unwrap_or_else(|| json!({}));
        if let Value::Object(ref mut map) = outcome_obj {
            map.entry("prediction".to_string())
                .or_insert_with(|| prediction.clone());
        }

        let contribution = json!({
            "schema_version": CONTRIBUTION_SCHEMA,
            "canon_version": CANON_VERSION,
            "mix_spec": mix,
            "process": process.cloned().unwrap_or_else(|| json!({})),
            "outcome": outcome_obj,
            "gate_summary": gate.gate_summary,
            "catalog_hash": DEFAULT_CATALOG_HASH,
            "observed_at": observed,
        });

        let (next, job_id) = self.contribute_async(profile, &contribution);
        Ok((
            next,
            json!({
                "job_id": job_id,
                "prediction": prediction,
                "gate_summary": gate.gate_summary,
            }),
        ))
    }

    /// MCP `umst_arena_open` — load arena bytes at the Warm boundary, return session id.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio transport; parse-once arena session for hot gate loops.
    pub fn arena_open(self, arena_path: &Path) -> Result<(Self, uuid::Uuid), String> {
        let bytes = std::fs::read(arena_path).map_err(|e| format!("arena read: {e}"))?;
        Self::arena_open_bytes(self, bytes)
    }

    /// Open arena session from owned bytes (validates header once via `load_arena`).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Warm-boundary parse-once; gate physics on `gate_check_arena` delegate.
    pub fn arena_open_bytes(self, bytes: Vec<u8>) -> Result<(Self, uuid::Uuid), String> {
        #[cfg(feature = "arena-session")]
        {
            umst_runtime_arena::load_arena(&bytes).map_err(|e| format!("arena parse: {e}"))?;
            let id = uuid::Uuid::new_v4();
            let mut sessions = self.arena_sessions;
            sessions.insert(id, Arc::from(bytes.into_boxed_slice()));
            Ok((
                Self {
                    arena_sessions: sessions,
                    ..self
                },
                id,
            ))
        }
        #[cfg(not(feature = "arena-session"))]
        {
            let _ = bytes;
            Err("umst-mcp built without arena-session feature".into())
        }
    }

    /// MCP `umst_gate_check_arena` — gate check against a warm arena session + mix wire.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Warm parse + in-process gate; no per-iter MCP round-trip.
    #[must_use]
    pub fn gate_check_arena(
        &self,
        profile: &Profile,
        arena_session_id: uuid::Uuid,
        mix: &Value,
        explain: bool,
    ) -> Result<GateCheckResult, String> {
        #[cfg(feature = "arena-session")]
        {
            let bytes = self
                .arena_sessions
                .get(&arena_session_id)
                .ok_or_else(|| format!("unknown arena_session_id: {arena_session_id}"))?;
            let view =
                umst_runtime_arena::load_arena(bytes).map_err(|e| format!("arena parse: {e}"))?;
            let mut result = self.gate_check(profile, mix, explain);
            let header = view.header();
            result
                .gate_summary
                .catalog_ids
                .push(format!("arena.abi_v{}", header.abi_version));
            result
                .gate_summary
                .catalog_ids
                .push(format!("arena.state_bytes.{}", header.state_bytes));
            Ok(result)
        }
        #[cfg(not(feature = "arena-session"))]
        {
            let _ = (profile, arena_session_id, mix, explain);
            Err("umst-mcp built without arena-session feature".into())
        }
    }

    /// MCP `umst_arena_close` — drop warm arena session bytes.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Session lifecycle; releases mmap-owned buffer handle.
    pub fn arena_close(self, arena_session_id: uuid::Uuid) -> Result<Self, String> {
        if !self.arena_sessions.contains_key(&arena_session_id) {
            return Err(format!("unknown arena_session_id: {arena_session_id}"));
        }
        let mut sessions = self.arena_sessions;
        sessions.remove(&arena_session_id);
        Ok(Self {
            arena_sessions: sessions,
            ..self
        })
    }
}

/// IO boundary: disk sidecar + JCS JSONL so `umst promote-contribution` can find rows later.
fn persist_memory_row(memory_id: &str, store: &ResearchStore) -> Result<(), String> {
    use std::fs;
    use std::path::PathBuf;

    let record = store
        .rows()
        .into_iter()
        .find(|r| r.memory_id.as_deref() == Some(memory_id))
        .ok_or_else(|| format!("memory row missing after accept: {memory_id}"))?;
    let dir = PathBuf::from(".umst-memory/rows");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{memory_id}.json"));
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())?;
    let _ = append_memory_jsonl(&record, None);
    Ok(())
}

/// Pinned JSON schema resources for MCP `resources/list`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Schema fixture bytes for MCP discovery; versioned wire only.
pub const SCHEMA_RESOURCES: &[(&str, &str)] = &[
    (
        "schemas/contribution.v1.json",
        include_str!("../../../schemas/contribution.v1.json"),
    ),
    (
        "schemas/memory_record.v1.json",
        include_str!("../../../schemas/memory_record.v1.json"),
    ),
    (
        "schemas/gate_reject.v1.json",
        include_str!("../../../schemas/gate_reject.v1.json"),
    ),
    (
        "schemas/observed_at.v2.json",
        include_str!("../../../schemas/observed_at.v2.json"),
    ),
    (
        "schemas/promotion_proposal.v1.json",
        include_str!("../../../schemas/promotion_proposal.v1.json"),
    ),
    (
        "schemas/promotion_approval.v1.json",
        include_str!("../../../schemas/promotion_approval.v1.json"),
    ),
    (
        "schemas/promotion_record.v1.json",
        include_str!("../../../schemas/promotion_record.v1.json"),
    ),
];

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP prompt templates; operational guidance only.
pub const AGENT_PROMPTS: &[(&str, &str, &str)] = &[
    (
        "safe-exploration",
        "Read-only agent exploration workflow",
        "Safe exploration: (1) umst_gate_check with explain:true (default) on candidate mix_spec; (2) optional umst_predict for constitutive detail; (3) umst_memory_query for similar admissible cases. Never call umst_contribute when gate_summary.admissible is false. On REJECT, read explain.regime_violations, explain.remediation, and explain.fields before adjusting the mix.",
    ),
    (
        "contribute-admissible",
        "Contribute an admissible mix observation",
        "Submit a contribution.v1 JSON with gate_summary.admissible=true. Run umst_gate_check first; never contribute REJECT mixes. Use rational strings for all physical quantities.",
    ),
    (
        "query-near-mix",
        "Query memory near a mix_spec anchor",
        "Call umst_memory_query with near_mix_spec (rational mix_spec), max_mix_l1 (e.g. 0.05), and admissible_only=true. Results are sorted by L1 distance in normalized mix space.",
    ),
    (
        "gate-before-contribute",
        "Hard gate workflow",
        "Always call umst_gate_check before umst_contribute. Rejects are appended to gate_reject.jcs.jsonl and never enter admissible_only memory.",
    ),
    (
        "interpret_gate_failure",
        "Interpret a gate REJECT for an agent",
        "When umst_gate_check returns isError with gate_reject.v1, read explain.regime_violations, explain.remediation, explain.fields, and catalog_witnesses. Typical codes: mix_spec_rational_parse_fail, thermodynamic_cd_fail. Apply remediation hints; re-run gate check before contribute.",
    ),
    (
        "suggest_similar_mix",
        "Find similar admissible mixes in memory",
        "Call umst_memory_query with near_mix_spec (full rational mix_spec), max_mix_l1 (start at 0.05), admissible_only=true, limit=10. Use cursor from next_cursor for pagination. Optionally add hilbert_index + max_hilbert_distance for Morton locality.",
    ),
    (
        "audit_mix_csv",
        "Audit a mix CSV batch against calibration",
        "Use umst_audit with dataset_d1-compatible CSV headers (cement, slag, fly_ash, water, superplasticizer, coarse_agg, fine_agg, age, strength, source, temperature, humidity). Set profile to match calibration bundle (e.g. uci_d1). Parse rows for regime warnings and abs_error_mpa before contributing validated outcomes.",
    ),
    (
        "export_for_git_inbox",
        "Export local memory rows for git contribution inbox PR",
        "After umst_gate_check → umst_contribute session: (1) export new admissible rows with `python3 scripts/export_contributions_jsonl.py --db $UMST_MEMORY_DB --lab <lab-slug> --out contributions/inbox/<lab-slug>-<YYYYMMDD>-<6char>.jsonl`; (2) validate locally with `python3 scripts/validate_contribution_inbox.py` on that file and `python3 scripts/ingest_contributions.py <file> --dry-run --skip-gate`; (3) open a PR adding only the inbox JSONL (one file per PR). Export skips content_ids already in contributions/merged/MANIFEST.jsonl. Maintainer merges → moves file to contributions/merged/YYYY-MM/ → appends manifest via `python3 scripts/update_contribution_manifest.py --append <path>`. MCP does not push to git.",
    ),
];

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP resource enumeration; schema bytes are versioned fixtures.
pub fn resources_list_result() -> Value {
    let resources: Vec<Value> = SCHEMA_RESOURCES
        .iter()
        .map(|(uri, _)| {
            json!({
                "uri": format!("umst://{uri}"),
                "name": uri,
                "mimeType": "application/json",
            })
        })
        .collect();
    json!({ "resources": resources })
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP resource read; returns pinned JSON schema text.
pub fn resources_read_result(uri: &str) -> Result<Value, String> {
    let key = uri.strip_prefix("umst://").unwrap_or(uri);
    SCHEMA_RESOURCES
        .iter()
        .find(|(path, _)| *path == key)
        .map(|(path, body)| {
            json!({
                "contents": [{
                    "uri": format!("umst://{path}"),
                    "mimeType": "application/json",
                    "text": body,
                }]
            })
        })
        .ok_or_else(|| format!("unknown resource: {uri}"))
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP prompts list; names only.
pub fn prompts_list_result() -> Value {
    let prompts: Vec<Value> = AGENT_PROMPTS
        .iter()
        .map(|(name, description, _)| {
            json!({
                "name": name,
                "description": description,
            })
        })
        .collect();
    json!({ "prompts": prompts })
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP prompt body fetch; operational text.
pub fn prompts_get_result(name: &str) -> Result<Value, String> {
    AGENT_PROMPTS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_n, description, body)| {
            json!({
                "description": description,
                "messages": [{
                    "role": "user",
                    "content": { "type": "text", "text": body },
                }],
            })
        })
        .ok_or_else(|| format!("unknown prompt: {name}"))
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP tool schema export; delegates to gate/memory/contribute impls.
pub fn agent_tools_schema() -> Vec<Value> {
    #[allow(unused_mut)] // mutated only when S7 feature cfgs append schemas
    let mut tools = vec![
        with_schema_2020(
            json!({
                "name": "umst_gate_check",
                "description": "Hard thermodynamic admissibility check (read-only). Always run before umst_contribute; never ingest when gate_summary.admissible is false. Example input: {\"mix\":{\"w_c\":\"9/20\",\"temperature_k\":\"29315/100\",\"aggregate_volume_fraction\":\"7/10\"},\"explain\":true}. Example output (PASS): {\"gate_summary\":{\"admissible\":true,\"verdict\":\"PASS\",\"catalog_ids\":[\"gate.clausius_duhem.v1\"]}}. On REJECT: isError:true with gate_reject.v1 + explain.regime_violations, explain.remediation, explain.fields (explain defaults true). Resource: umst://schemas/gate_reject.v1.json.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mix": { "type": "object", "description": "mix_spec.v1 rational fields — e.g. {\"w_c\":\"9/20\",\"temperature_k\":\"29315/100\",\"aggregate_volume_fraction\":\"7/10\"}; use \"n/d\" strings, never JSON floats" },
                        "profile": { "type": "string", "default": "default", "description": "Bundled calibration profile id (call umst_profiles when unsure)" },
                        "explain": { "type": "boolean", "default": true, "description": "When true (default), include regime_violations, remediation, fields, and catalog_witnesses on REJECT" }
                    },
                    "required": ["mix"]
                }
            }),
            true,
        ),
        with_schema_2020(
            json!({
                "name": "umst_contribute",
                "description": "Gate-validated contribution ingest → local research memory (admissible rows only; destructive). Requires prior umst_gate_check PASS and contribution.v1 with matching gate_summary.admissible:true. Validate wire via umst://schemas/contribution.v1.json. Example input: {\"contribution\":{\"schema_version\":\"contribution.v1\",\"mix_spec\":{\"w_c\":\"9/20\",...},\"gate_summary\":{\"admissible\":true,\"verdict\":\"PASS\"},...}}. Example output: {\"memory_id\":\"<uuid>\",\"content_id\":\"sha256:...\",\"observed_at\":{\"stamp_tier\":\"Synthetic\",...}}. Supports idempotency_key; set async:true for job_id + umst_contribute_status poll. scope_token required when UMST_AGENT_SCOPE_TOKENS is set.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "contribution": { "type": "object", "description": "contribution.v1 JSON — see umst://schemas/contribution.v1.json; gate_summary.admissible must be true" },
                        "profile": { "type": "string", "default": "default", "description": "Bundled calibration profile id used for gate re-check on ingest" },
                        "scope_token": { "type": "string", "description": "Required when UMST_AGENT_SCOPE_TOKENS is set; must match an allowed token" },
                        "async": { "type": "boolean", "description": "When true, return job_id immediately; poll umst_contribute_status until SUCCEEDED or FAILED" }
                    },
                    "required": ["contribution"]
                }
            }),
            false,
        ),
        with_schema_2020(
            json!({
                "name": "umst_contribute_status",
                "description": "Poll async contribute job state. Jobs persist in contribute_jobs.json beside UMST_MEMORY_DB when set.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "job_id": { "type": "string" }
                    },
                    "required": ["job_id"]
                }
            }),
            true,
        ),
        with_schema_2020(
            json!({
                "name": "umst_memory_query",
                "description": "Paginated filter on gate-passed memory: regime, catalog_id, stamp_tier, outcome.source, wall_ms window, mix L1 (near_mix_spec + max_mix_l1), or Morton hilbert_index + max_hilbert_distance.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "admissible_only": { "type": "boolean", "default": true },
                        "curing_regime": { "type": "string" },
                        "catalog_id": { "type": "string", "description": "Filter rows whose catalog_ids contains this witness" },
                        "stamp_tier": { "type": "string", "description": "observed_at.stamp_tier (e.g. Synthetic, UcrsTier2)" },
                        "outcome_source": { "type": "string", "description": "payload.outcome.source field" },
                        "wall_ms_min": { "type": "integer", "minimum": 0 },
                        "wall_ms_max": { "type": "integer", "minimum": 0 },
                        "cursor": { "type": "string", "description": "content_id from prior page next_cursor" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 500, "default": 50 },
                        "near_mix_spec": { "type": "object", "description": "Anchor mix_spec for L1 distance sort/filter" },
                        "max_mix_l1": { "type": "number", "description": "Max L1 distance from near_mix_spec anchor" },
                        "hilbert_index": { "type": "integer", "minimum": 0, "description": "Morton bucket for locality query" },
                        "max_hilbert_distance": { "type": "integer", "minimum": 0, "description": "Max Morton distance from hilbert_index (default 0 = exact)" }
                    }
                }
            }),
            true,
        ),
        with_schema_2020(
            json!({
                "name": "umst_mi_estimate",
                "description": "Advisory MI bits estimate (Landauer envelope surrogate; not an admissibility gate).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mix": { "type": "object", "description": "mix_spec.v1 rational fields" }
                    },
                    "required": ["mix"]
                }
            }),
            true,
        ),
        with_schema_2020(
            json!({
                "name": "umst_transition_propose",
                "description": "Predict constitutive scalars, hard-gate the mix, then enqueue async contribute. Returns job_id for umst_contribute_status.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mix": { "type": "object", "description": "mix_spec.v1 rational fields" },
                        "profile": { "type": "string", "default": "default" },
                        "outcome": { "type": "object", "description": "Optional outcome.v1 fields; prediction merged when absent" },
                        "process": { "type": "object", "description": "Optional process metadata (curing_regime, etc.)" }
                    },
                    "required": ["mix"]
                }
            }),
            false,
        ),
        with_schema_2020(
            json!({
                "name": "umst_arena_open",
                "description": "Load arena bytes at the Warm boundary and return arena_session_id for hot gate loops. Validates header via load_arena once. Example: {\"arena_path\":\"fixtures/arena/minimal.v1.bin\"}.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "arena_path": { "type": "string", "description": "Path to versioned arena file (ABI v1)" }
                    },
                    "required": ["arena_path"]
                }
            }),
            false,
        ),
        with_schema_2020(
            json!({
                "name": "umst_gate_check_arena",
                "description": "Gate check using a warm arena session (parse-once) plus mix wire. Same physics as umst_gate_check; adds arena catalog witnesses.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "arena_session_id": { "type": "string", "description": "UUID from umst_arena_open" },
                        "mix": { "type": "object", "description": "mix_spec.v1 rational fields" },
                        "profile": { "type": "string", "default": "default" },
                        "explain": { "type": "boolean", "default": true }
                    },
                    "required": ["arena_session_id", "mix"]
                }
            }),
            true,
        ),
        with_schema_2020(
            json!({
                "name": "umst_arena_close",
                "description": "Release warm arena session bytes. Idempotent drop of arena_session_id from MCP session map.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "arena_session_id": { "type": "string" }
                    },
                    "required": ["arena_session_id"]
                }
            }),
            false,
        ),
    ];
    #[cfg(feature = "gate-explain-v2")]
    crate::proposed_tools::patch_gate_check_schema(&mut tools);
    #[cfg(any(
        feature = "tool-dry-run",
        feature = "tool-promote",
        feature = "tool-arena-session-unified"
    ))]
    tools.extend(crate::proposed_tools::proposed_tool_schemas());
    tools
}
