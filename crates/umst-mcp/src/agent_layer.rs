// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! MCP Physical Reasoning Layer — **pure session threading** at the impure stdio boundary.

use serde_json::{json, Value};
use std::collections::HashMap;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{
    accept, append_gate_reject_jsonl, append_memory_jsonl, estimate_mi_bits_from_mix,
    gate_check_mix, gate_reject_row_for_mix, query, synthetic_observed_at, AcceptError,
    AcceptResult, GateContext, GateSummary, MemoryQuery, MemoryRecord, ProvenanceClock,
    ResearchStore, WallClock,
};

/// Async contribute job state (in-memory stub for heavy physics path).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP job status wire; physics on `gate_check_mix` / `accept`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
#[derive(Debug, Clone, serde::Serialize)]
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
}

impl Default for AgentSession {
    fn default() -> Self {
        Self {
            store: ResearchStore::from_env().unwrap_or_default(),
            clock: ProvenanceClock::default(),
            jobs: HashMap::new(),
        }
    }
}

impl AgentSession {
    /// MCP `umst_gate_check` — delegates to cartridge `gate_check_mix`.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio JSON-RPC transport; CD admissibility on `gate_check_mix`.
    #[must_use]
    pub fn gate_check(&self, profile: &Profile, mix: &Value) -> GateSummary {
        let summary = gate_check_mix(profile, mix);
        if !summary.admissible {
            let observed = synthetic_observed_at(self.clock.sequence());
            if let Some(row) = gate_reject_row_for_mix(mix, &summary, observed) {
                let _ = append_gate_reject_jsonl(&row, None);
            }
        }
        summary
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
    ) -> Result<(Self, AcceptResult), String> {
        let ctx = GateContext { profile };
        match accept(self.store, self.clock, WallClock, &ctx, contribution) {
            Ok((store, clock, result)) => {
                let session = Self {
                    store: store.clone(),
                    clock,
                    jobs: self.jobs,
                };
                persist_memory_row(&result.memory_id, &store)?;
                Ok((session, result))
            }
            Err(AcceptError::GateReject(row)) => {
                let _ = append_gate_reject_jsonl(&row, None);
                Err("gate re-check failed: mix not thermodynamically admissible".into())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// MCP async contribute stub — returns job id immediately.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: In-process async wrapper; same gate path as `contribute`.
    pub fn contribute_async(
        mut self,
        profile: &Profile,
        contribution: &Value,
    ) -> (Self, String) {
        let job_id = uuid::Uuid::new_v4().to_string();
        self.jobs.insert(
            job_id.clone(),
            ContributeJob {
                job_id: job_id.clone(),
                status: ContributeJobStatus::Running,
                result: None,
                error: None,
            },
        );
        let profile = profile.clone();
        let contribution = contribution.clone();
        match self.clone().contribute(&profile, &contribution) {
            Ok((session, result)) => {
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
                (Self { jobs, ..session }, job_id)
            }
            Err(e) => {
                self.jobs.insert(
                    job_id.clone(),
                    ContributeJob {
                        job_id: job_id.clone(),
                        status: ContributeJobStatus::Failed,
                        result: None,
                        error: Some(e),
                    },
                );
                (self, job_id)
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

    /// MCP `umst_memory_query` — pure filter over session store.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: stdio transport over cartridge `query`; filter on `filter_records`.
    #[must_use]
    pub fn memory_query(&self, q: &MemoryQuery) -> Vec<MemoryRecord> {
        query(&self.store, q)
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
    vec![
        json!({
            "name": "umst_gate_check",
            "description": "Hard admissibility verdict + catalog_ids + optional mi_bits_est for a mix_spec (manifest CD when agent-layer built). Rejects append to gate_reject.jcs.jsonl.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": { "type": "object", "description": "mix_spec.v1 rational fields" },
                    "profile": { "type": "string", "default": "default" }
                },
                "required": ["mix"]
            }
        }),
        json!({
            "name": "umst_contribute",
            "description": "Gate-validated contribution ingest → local research memory (admissible rows only). Supports idempotency_key.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "contribution": { "type": "object", "description": "contribution.v1 JSON" },
                    "profile": { "type": "string", "default": "default" },
                    "scope_token": { "type": "string", "description": "Required when UMST_AGENT_SCOPE_TOKENS is set" },
                    "async": { "type": "boolean", "description": "Return job_id for umst_contribute_status when true" }
                },
                "required": ["contribution"]
            }
        }),
        json!({
            "name": "umst_contribute_status",
            "description": "Poll async contribute job state (in-memory stub for heavy physics path).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "job_id": { "type": "string" }
                },
                "required": ["job_id"]
            }
        }),
        json!({
            "name": "umst_memory_query",
            "description": "Filter gate-passed memory by regime, mix L1 distance, or Morton hilbert_index locality.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "admissible_only": { "type": "boolean", "default": true },
                    "curing_regime": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1 },
                    "near_mix_spec": { "type": "object", "description": "Anchor mix_spec for L1 distance" },
                    "max_mix_l1": { "type": "number", "description": "Max L1 distance from anchor" },
                    "hilbert_index": { "type": "integer", "minimum": 0 },
                    "max_hilbert_distance": { "type": "integer", "minimum": 0 }
                }
            }
        }),
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
    ]
}
