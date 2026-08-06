// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// Product spine honesty probe — documents what is wired vs stub without claiming closure.
//
// **Policy:** `mcp_product_spine_honest()` stays true while `top_level_spine_present` is false
// and all functional stubs remain honestly open. No PASS/GREEN/OP-5 invent.
//
// **FLEET-COMPOSER-D D23 (1741):** Wave-4 umst-mcp spine gaps audit — typed D23 probe folding
// C13/C37 receipt chain + facet census + gateway boundary honesty. No facet closure flip.
//
// **FLEET-COMPOSER-F F68 (1934):** Wave-5 MCP schema spine deepen — folds D25 product spine
// census + gateway W-09 schema receipt chain. No facet closure flip.
//
// **FLEET-COMPOSER-H H08 (2242):** Wave-H native stdio JSON-RPC smoke harden — folds F68 +
// `stdio_smoke` 4-slot battery + F11/G39 authority chain. No production flip.

/// FLEET-COMPOSER-D D23 card id (1741 fleet Wave 4).
pub const COMPOSER_D23_JOB_ID: &str = "FLEET-COMPOSER-D23";

/// FLEET-COMPOSER-D D23 completion receipt cross-ref.
pub const COMPOSER_D23_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_D23_MCP_SPINE_1741.md";

/// FLEET-COMPOSER-D D23 wave slot number.
pub const COMPOSER_D23_WAVE_SLOT: &str = "23";

/// FLEET-COMPOSER-D manifest cross-ref.
pub const FLEET_D_MANIFEST_PATH: &str = "outputs/.tmp/FLEET_COMPOSER_D_100_1741.md";

/// Model slug stamped on FLEET-COMPOSER deepen (receipt attribution).
pub const COMPOSER_MODEL_SLUG: &str = "composer-2.5";

/// FLEET-COMPOSER-C C13 spine audit receipt cross-ref.
pub const COMPOSER_C13_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_C13_MCP_SPINE_1649.md";

/// FLEET-COMPOSER-C C37 Round-2 MCP gaps absorb receipt cross-ref.
pub const COMPOSER_C37_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_C37_R2_MCP_GAPS_1649.md";

/// Round-2 Kimi baseline — `outputs/.tmp` copy may be missing; absorbed by C37.
pub const ROUND2_RECEIPT_PATH: &str = "outputs/.tmp/UMST_MCP_STUBS_ROUND2_1514.md";

/// Round-2 Wave 1 planned gaps receipt — never written; absorbed by C37 + D25.
pub const ROUND2_GAPS_RECEIPT_PATH: &str = "outputs/.tmp/UMST_MCP_GAPS_ROUND2_1514.md";

/// FLEET-COMPOSER-D D25 card id (1741 fleet Wave 4 Round-2 backfill).
pub const COMPOSER_D25_JOB_ID: &str = "FLEET-COMPOSER-D25";

/// FLEET-COMPOSER-D D25 Round-2 MCP gaps absorb receipt cross-ref.
pub const COMPOSER_D25_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_D25_R2_MCP_GAPS_1741.md";

/// FLEET-COMPOSER-D D25 wave slot number.
pub const COMPOSER_D25_WAVE_SLOT: &str = "25";

/// FLEET-COMPOSER-F F68 card id (1934 fleet Wave 5).
pub const COMPOSER_F68_JOB_ID: &str = "FLEET-COMPOSER-F68-MCP-SCHEMA";

/// FLEET-COMPOSER-F F68 completion receipt cross-ref.
pub const COMPOSER_F68_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F68_MCP_1934.md";

/// FLEET-COMPOSER-F F68 wave slot number.
pub const COMPOSER_F68_WAVE_SLOT: &str = "F68";

/// FLEET-COMPOSER-F manifest cross-ref.
pub const FLEET_F_MANIFEST_PATH: &str = "outputs/.tmp/FLEET_COMPOSER_F_100_1940.md";

/// Gateway W-09 schema deepen receipt cross-ref (external boundary).
pub const GATEWAY_MCP_SCHEMA_F68_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F68_MCP_1934.md";

/// Round-2 job id for umst-mcp product spine gaps (Wave 1 Kimi slot 19).
pub const ROUND2_MCP_GAPS_JOB_ID: &str = "UMST-MCP-GAPS-ROUND2";

/// Canonical crate path inside the monorepo (host workspace).
pub const CANONICAL_CRATE_PATH: &str = "umst-concrete-cartridge/crates/umst-mcp";

/// Top-level `umst-mcp/` workspace root — locator README only until promotion.
pub const TOP_LEVEL_SPINE_LOCATOR: &str = "umst-mcp/README.md";

/// Whether a standalone top-level `umst-mcp/` workspace exists (crate promotion).
pub const fn top_level_spine_present() -> bool {
    false
}

/// Constitutional 13-tool GO-LIVE surface (`default = ["agent-layer"]`).
pub const fn constitutional_tools_wired() -> bool {
    true
}

/// S7c `umst_promote_contribution` — returns `promote_not_wired`.
#[cfg(feature = "tool-promote")]
pub const fn promote_contribution_wired() -> bool {
    false
}

/// S7c disabled in default build — not exposed on `tools/list`.
#[cfg(not(feature = "tool-promote"))]
pub const fn promote_contribution_wired() -> bool {
    false
}

/// HCOM-029 `refine_shape` — honest stub until HCOM-020 Kleisli wire.
#[cfg(feature = "tool-propose-communicative-act")]
pub const fn refine_shape_wired() -> bool {
    false
}

#[cfg(not(feature = "tool-propose-communicative-act"))]
pub const fn refine_shape_wired() -> bool {
    false
}

/// WEB-009 `web.propose_delta` — mock fold only (`exec_web_propose_delta_mock`).
#[cfg(feature = "tool-web-propose-delta")]
pub const fn web_propose_delta_live_fold_wired() -> bool {
    false
}

#[cfg(not(feature = "tool-web-propose-delta"))]
pub const fn web_propose_delta_live_fold_wired() -> bool {
    false
}

/// Async contribute job queue — in-memory / sidecar JSONL; not durable (TODO-M3-006 class).
pub const fn durable_contribute_queue_wired() -> bool {
    false
}

/// Gateway L5 native MCP wrap closure — owned by `umst-gateway` (`R-gateway-wrap-native-mcp`).
pub const fn gateway_native_wrap_closed() -> bool {
    false
}

/// One row on the honest MCP product spine census.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpSpineFacet {
    /// Facet id.
    pub id: &'static str,
    /// Whether production wiring is claimed.
    pub wired: bool,
    /// Owning residual when open.
    pub residue: &'static str,
}

/// Honest product spine probe — Partial max; no closure invent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineProbe {
    /// Canonical nested crate path.
    pub canonical_crate_path: &'static str,
    /// Top-level spine locator doc path.
    pub top_level_spine_locator: &'static str,
    /// Whether standalone top-level workspace exists.
    pub top_level_spine_present: bool,
    /// Constitutional 13-tool surface wired.
    pub constitutional_tools_wired: bool,
    /// `umst_promote_contribution` production wired.
    pub promote_contribution_wired: bool,
    /// `refine_shape` Kleisli wire closed.
    pub refine_shape_wired: bool,
    /// `web.propose_delta` live wasm fold wired.
    pub web_propose_delta_live_fold_wired: bool,
    /// Durable async contribute queue wired.
    pub durable_contribute_queue_wired: bool,
    /// Gateway native MCP wrap closed (external boundary).
    pub gateway_native_wrap_closed: bool,
}

/// Build the honest MCP product spine probe for the current feature profile.
#[must_use]
pub fn mcp_product_spine_probe() -> McpProductSpineProbe {
    McpProductSpineProbe {
        canonical_crate_path: CANONICAL_CRATE_PATH,
        top_level_spine_locator: TOP_LEVEL_SPINE_LOCATOR,
        top_level_spine_present: top_level_spine_present(),
        constitutional_tools_wired: constitutional_tools_wired(),
        promote_contribution_wired: promote_contribution_wired(),
        refine_shape_wired: refine_shape_wired(),
        web_propose_delta_live_fold_wired: web_propose_delta_live_fold_wired(),
        durable_contribute_queue_wired: durable_contribute_queue_wired(),
        gateway_native_wrap_closed: gateway_native_wrap_closed(),
    }
}

/// Facet inventory for operator census tables.
#[must_use]
pub fn mcp_spine_facet_inventory() -> [McpSpineFacet; 7] {
    let probe = mcp_product_spine_probe();
    [
        McpSpineFacet {
            id: "top_level_spine",
            wired: probe.top_level_spine_present,
            residue: "umst-mcp/README.md locator only",
        },
        McpSpineFacet {
            id: "constitutional_13_tools",
            wired: probe.constitutional_tools_wired,
            residue: "GO-LIVE Step 3",
        },
        McpSpineFacet {
            id: "promote_contribution",
            wired: probe.promote_contribution_wired,
            residue: "S7c tool-promote stub",
        },
        McpSpineFacet {
            id: "refine_shape",
            wired: probe.refine_shape_wired,
            residue: "HCOM-020 Kleisli",
        },
        McpSpineFacet {
            id: "web_propose_delta_live",
            wired: probe.web_propose_delta_live_fold_wired,
            residue: "1836-spawn WEB-009",
        },
        McpSpineFacet {
            id: "durable_contribute_queue",
            wired: probe.durable_contribute_queue_wired,
            residue: "TODO-M3-006",
        },
        McpSpineFacet {
            id: "gateway_native_wrap",
            wired: probe.gateway_native_wrap_closed,
            residue: "R-gateway-wrap-native-mcp",
        },
    ]
}

/// Honest spine census — true while locator exists and no facet falsely claims closure.
#[must_use]
pub fn mcp_product_spine_honest(probe: &McpProductSpineProbe) -> bool {
    !probe.top_level_spine_present
        && probe.constitutional_tools_wired
        && !probe.promote_contribution_wired
        && !probe.refine_shape_wired
        && !probe.web_propose_delta_live_fold_wired
        && !probe.durable_contribute_queue_wired
        && !probe.gateway_native_wrap_closed
}

/// FLEET-COMPOSER-D D23 typed probe — folds spine census + receipt authority chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineD23Probe {
    /// FLEET-COMPOSER-D23 card id.
    pub composer_d23_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// D23 wave slot.
    pub composer_d23_wave_slot: &'static str,
    /// Base spine census probe.
    pub spine: McpProductSpineProbe,
    /// Base spine honesty gate.
    pub spine_honest: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
    /// Wired facet count (constitutional only at D23).
    pub wired_facet_count: usize,
    /// Open facet count.
    pub open_facet_count: usize,
}

/// Receipt cross-ref chain for D23 audit (C13 + C37 + Round-2 archived baseline).
#[must_use]
pub fn mcp_product_spine_d23_authority_chain_honest() -> bool {
    COMPOSER_D23_RECEIPT_PATH.contains("COMPOSER_D23_MCP_SPINE_1741")
        && COMPOSER_C13_RECEIPT_PATH.contains("COMPOSER_C13_MCP_SPINE_1649")
        && COMPOSER_C37_RECEIPT_PATH.contains("COMPOSER_C37_R2_MCP_GAPS_1649")
        && ROUND2_RECEIPT_PATH.contains("UMST_MCP_STUBS_ROUND2_1514")
        && FLEET_D_MANIFEST_PATH.contains("FLEET_COMPOSER_D_100_1741")
}

/// Build the FLEET-COMPOSER-D D23 spine audit probe.
#[must_use]
pub fn mcp_product_spine_d23_probe() -> McpProductSpineD23Probe {
    let spine = mcp_product_spine_probe();
    let facets = mcp_spine_facet_inventory();
    let wired_facet_count = facets.iter().filter(|f| f.wired).count();
    let open_facet_count = facets.iter().filter(|f| !f.wired).count();
    McpProductSpineD23Probe {
        composer_d23_job_id: COMPOSER_D23_JOB_ID,
        composer_model_slug: COMPOSER_MODEL_SLUG,
        composer_d23_wave_slot: COMPOSER_D23_WAVE_SLOT,
        spine,
        spine_honest: mcp_product_spine_honest(&spine),
        authority_chain_honest: mcp_product_spine_d23_authority_chain_honest(),
        wired_facet_count,
        open_facet_count,
    }
}

/// D23 honesty gate — partial max; no facet closure invent.
#[must_use]
pub fn mcp_product_spine_d23_honest(probe: &McpProductSpineD23Probe) -> bool {
    probe.composer_d23_job_id == COMPOSER_D23_JOB_ID
        && probe.composer_model_slug == COMPOSER_MODEL_SLUG
        && probe.composer_d23_wave_slot == COMPOSER_D23_WAVE_SLOT
        && probe.spine_honest
        && mcp_product_spine_honest(&probe.spine)
        && probe.authority_chain_honest
        && mcp_product_spine_d23_authority_chain_honest()
        && probe.wired_facet_count == 1
        && probe.open_facet_count == 6
        && !probe.spine.gateway_native_wrap_closed
        && !probe.spine.top_level_spine_present
}

/// FLEET-COMPOSER-D D25 typed probe — Round-2 umst-mcp gaps absorb backfill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineD25Probe {
    /// FLEET-COMPOSER-D25 card id.
    pub composer_d25_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// D25 wave slot.
    pub composer_d25_wave_slot: &'static str,
    /// Round-2 job id (Wave 1 Kimi slot 19).
    pub round2_job_id: &'static str,
    /// Planned Round-2 gaps receipt path (missing on disk).
    pub round2_gaps_receipt_path: &'static str,
    /// C37 absorb receipt path (landed).
    pub round2_absorbed_receipt_path: &'static str,
    /// D25 completion receipt path.
    pub composer_d25_receipt_path: &'static str,
    /// D23 spine audit probe folded into D25 chain.
    pub d23: McpProductSpineD23Probe,
    /// Round-2 absorb chain honest.
    pub round2_absorb_honest: bool,
    /// D-fleet Round-2 MCP gaps absorb honest.
    pub d_fleet_round2_absorb_honest: bool,
}

/// Round-2 umst-mcp gaps absorb honesty — missing 1514 gaps receipt acknowledged; C37 chain landed.
#[must_use]
pub fn mcp_product_spine_round2_absorb_honest() -> bool {
    ROUND2_MCP_GAPS_JOB_ID == "UMST-MCP-GAPS-ROUND2"
        && ROUND2_GAPS_RECEIPT_PATH.contains("UMST_MCP_GAPS_ROUND2_1514")
        && COMPOSER_C37_RECEIPT_PATH.contains("COMPOSER_C37_R2_MCP_GAPS_1649")
        && ROUND2_RECEIPT_PATH.contains("UMST_MCP_STUBS_ROUND2_1514")
}

/// D-fleet Round-2 MCP gaps absorb chain — C37 + D25 without facet closure flip.
#[must_use]
pub fn mcp_product_spine_d_fleet_round2_absorb_honest() -> bool {
    mcp_product_spine_round2_absorb_honest()
        && COMPOSER_D25_RECEIPT_PATH.contains("COMPOSER_D25_R2_MCP_GAPS_1741")
        && COMPOSER_D25_JOB_ID == "FLEET-COMPOSER-D25"
        && FLEET_D_MANIFEST_PATH.contains("FLEET_COMPOSER_D_100_1741")
}

/// Build the FLEET-COMPOSER-D D25 Round-2 absorb probe.
#[must_use]
pub fn mcp_product_spine_d25_probe() -> McpProductSpineD25Probe {
    McpProductSpineD25Probe {
        composer_d25_job_id: COMPOSER_D25_JOB_ID,
        composer_model_slug: COMPOSER_MODEL_SLUG,
        composer_d25_wave_slot: COMPOSER_D25_WAVE_SLOT,
        round2_job_id: ROUND2_MCP_GAPS_JOB_ID,
        round2_gaps_receipt_path: ROUND2_GAPS_RECEIPT_PATH,
        round2_absorbed_receipt_path: COMPOSER_C37_RECEIPT_PATH,
        composer_d25_receipt_path: COMPOSER_D25_RECEIPT_PATH,
        d23: mcp_product_spine_d23_probe(),
        round2_absorb_honest: mcp_product_spine_round2_absorb_honest(),
        d_fleet_round2_absorb_honest: mcp_product_spine_d_fleet_round2_absorb_honest(),
    }
}

/// D25 honesty gate — partial max; Round-2 backfill without facet closure invent.
#[must_use]
pub fn mcp_product_spine_d25_honest(probe: &McpProductSpineD25Probe) -> bool {
    probe.composer_d25_job_id == COMPOSER_D25_JOB_ID
        && probe.composer_model_slug == COMPOSER_MODEL_SLUG
        && probe.composer_d25_wave_slot == COMPOSER_D25_WAVE_SLOT
        && probe.round2_job_id == ROUND2_MCP_GAPS_JOB_ID
        && probe.round2_gaps_receipt_path.contains("UMST_MCP_GAPS_ROUND2_1514")
        && probe.round2_absorbed_receipt_path.contains("COMPOSER_C37_R2_MCP_GAPS_1649")
        && probe.composer_d25_receipt_path.contains("COMPOSER_D25_R2_MCP_GAPS_1741")
        && probe.d23.spine_honest
        && mcp_product_spine_d23_honest(&probe.d23)
        && probe.round2_absorb_honest
        && mcp_product_spine_round2_absorb_honest()
        && probe.d_fleet_round2_absorb_honest
        && mcp_product_spine_d_fleet_round2_absorb_honest()
        && probe.d23.wired_facet_count == 1
        && probe.d23.open_facet_count == 6
        && !probe.d23.spine.gateway_native_wrap_closed
}

/// F68 gateway schema receipt cross-ref honesty (external umst-gateway boundary).
#[must_use]
pub fn mcp_product_spine_f68_gateway_schema_receipt_honest() -> bool {
    GATEWAY_MCP_SCHEMA_F68_RECEIPT_PATH.contains("COMPOSER_F68_MCP_1934")
        && COMPOSER_F68_RECEIPT_PATH.contains("COMPOSER_F68_MCP_1934")
}

/// F68 receipt authority chain — F68 + D25 + D23 + F manifest.
#[must_use]
pub fn mcp_product_spine_f68_authority_chain_honest() -> bool {
    COMPOSER_F68_RECEIPT_PATH.contains("COMPOSER_F68_MCP_1934")
        && FLEET_F_MANIFEST_PATH.contains("FLEET_COMPOSER_F_100_1940")
        && COMPOSER_D25_RECEIPT_PATH.contains("COMPOSER_D25_R2_MCP_GAPS_1741")
        && COMPOSER_D23_RECEIPT_PATH.contains("COMPOSER_D23_MCP_SPINE_1741")
        && mcp_product_spine_d23_authority_chain_honest()
}

/// FLEET-COMPOSER-F F68 typed probe — product spine + gateway schema receipt fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineF68Probe {
    /// FLEET-COMPOSER-F68 card id.
    pub composer_f68_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// F68 wave slot.
    pub composer_f68_wave_slot: &'static str,
    /// D25 Round-2 absorb probe folded into F68.
    pub d25: McpProductSpineD25Probe,
    /// Gateway W-09 schema receipt cross-ref honest.
    pub gateway_schema_receipt_honest: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
    /// Wired facet count (constitutional only).
    pub wired_facet_count: usize,
    /// Open facet count.
    pub open_facet_count: usize,
}

/// Build the FLEET-COMPOSER-F F68 MCP schema spine deepen probe.
#[must_use]
pub fn mcp_product_spine_f68_probe() -> McpProductSpineF68Probe {
    let facets = mcp_spine_facet_inventory();
    let wired_facet_count = facets.iter().filter(|f| f.wired).count();
    let open_facet_count = facets.iter().filter(|f| !f.wired).count();
    McpProductSpineF68Probe {
        composer_f68_job_id: COMPOSER_F68_JOB_ID,
        composer_model_slug: COMPOSER_MODEL_SLUG,
        composer_f68_wave_slot: COMPOSER_F68_WAVE_SLOT,
        d25: mcp_product_spine_d25_probe(),
        gateway_schema_receipt_honest: mcp_product_spine_f68_gateway_schema_receipt_honest(),
        authority_chain_honest: mcp_product_spine_f68_authority_chain_honest(),
        wired_facet_count,
        open_facet_count,
    }
}

/// F68 honesty gate — partial max; schema spine deepen without facet closure invent.
#[must_use]
pub fn mcp_product_spine_f68_honest(probe: &McpProductSpineF68Probe) -> bool {
    probe.composer_f68_job_id == COMPOSER_F68_JOB_ID
        && probe.composer_model_slug == COMPOSER_MODEL_SLUG
        && probe.composer_f68_wave_slot == COMPOSER_F68_WAVE_SLOT
        && mcp_product_spine_d25_honest(&probe.d25)
        && probe.gateway_schema_receipt_honest
        && mcp_product_spine_f68_gateway_schema_receipt_honest()
        && probe.authority_chain_honest
        && mcp_product_spine_f68_authority_chain_honest()
        && probe.wired_facet_count == 1
        && probe.open_facet_count == 6
        && !probe.d25.d23.spine.gateway_native_wrap_closed
        && !probe.d25.d23.spine.top_level_spine_present
}

/// FLEET-COMPOSER-H H08 typed probe — folds F68 schema spine + native stdio smoke battery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineH08Probe {
    /// FLEET-COMPOSER-H08 card id.
    pub composer_h08_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// H08 wave slot.
    pub composer_h08_wave_slot: &'static str,
    /// F68 schema spine probe folded into H08.
    pub f68: McpProductSpineF68Probe,
    /// Native stdio smoke battery slot count.
    pub stdio_smoke_slot_count: usize,
    /// Stdio subprocess smoke reproducible via cargo test.
    pub stdio_smoke_reproducible: bool,
    /// WEB-009 production closed (honest false).
    pub web_009_production_closed: bool,
    /// Native stdio production wired (honest false).
    pub native_stdio_smoke_production_wired: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
}

/// H08 receipt authority chain (F11 + G39 + H manifest + stdio smoke module).
#[must_use]
pub fn mcp_product_spine_h08_authority_chain_honest() -> bool {
    crate::stdio_smoke::PRIOR_F11_RECEIPT_PATH.contains("COMPOSER_F11_STDIO_1942")
        && crate::stdio_smoke::PRIOR_G39_RECEIPT_PATH.contains("COMPOSER_G39_MCP_WRAP_2143")
        && crate::stdio_smoke::FLEET_H_MANIFEST_PATH.contains("FLEET_COMPOSER_H_100_2242")
        && crate::stdio_smoke::COMPOSER_H08_RECEIPT_PATH.contains("COMPOSER_H08_2242")
        && crate::stdio_smoke::native_stdio_smoke_h08_authority_chain_honest()
}

/// Build the FLEET-COMPOSER-H H08 native stdio smoke deepen probe.
#[must_use]
pub fn mcp_product_spine_h08_probe() -> McpProductSpineH08Probe {
    McpProductSpineH08Probe {
        composer_h08_job_id: crate::stdio_smoke::COMPOSER_H08_JOB_ID,
        composer_model_slug: COMPOSER_MODEL_SLUG,
        composer_h08_wave_slot: crate::stdio_smoke::COMPOSER_H08_WAVE_SLOT,
        f68: mcp_product_spine_f68_probe(),
        stdio_smoke_slot_count: crate::stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT,
        stdio_smoke_reproducible: crate::stdio_smoke::native_stdio_smoke_reproducible(),
        web_009_production_closed: crate::stdio_smoke::web_009_production_closed(),
        native_stdio_smoke_production_wired: crate::stdio_smoke::native_stdio_smoke_production_wired(),
        authority_chain_honest: mcp_product_spine_h08_authority_chain_honest(),
    }
}

/// H08 honesty gate — partial max; stdio GREEN without production flip invent.
#[must_use]
pub fn mcp_product_spine_h08_honest(probe: &McpProductSpineH08Probe) -> bool {
    probe.composer_h08_job_id == crate::stdio_smoke::COMPOSER_H08_JOB_ID
        && probe.composer_model_slug == COMPOSER_MODEL_SLUG
        && probe.composer_h08_wave_slot == crate::stdio_smoke::COMPOSER_H08_WAVE_SLOT
        && mcp_product_spine_f68_honest(&probe.f68)
        && probe.stdio_smoke_slot_count == crate::stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT
        && probe.stdio_smoke_reproducible
        && !probe.web_009_production_closed
        && !probe.native_stdio_smoke_production_wired
        && probe.authority_chain_honest
        && mcp_product_spine_h08_authority_chain_honest()
        && !probe.f68.d25.d23.spine.gateway_native_wrap_closed
}

/// FLEET-COMPOSER-X X05 typed probe — folds H08 stdio smoke + WEB-009 retick boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpProductSpineX05Probe {
    /// FLEET-COMPOSER-X05 card id.
    pub composer_x05_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// X05 wave slot.
    pub composer_x05_wave_slot: &'static str,
    /// H08 stdio smoke probe folded into X05.
    pub h08: McpProductSpineH08Probe,
    /// WEB-009 stdio smoke retick probe.
    pub web_009: crate::web_009::Web009StdioSmokeX05Probe,
    /// Native stdio smoke battery slot count.
    pub stdio_smoke_slot_count: usize,
    /// Stdio subprocess smoke reproducible via cargo test.
    pub stdio_smoke_reproducible: bool,
    /// WEB-009 production closed (honest false).
    pub web_009_production_closed: bool,
    /// WEB-009 stdio production wired (honest false).
    pub web_009_stdio_production_wired: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
}

/// X05 receipt authority chain (H08 + X manifest + web_009 module).
#[must_use]
pub fn mcp_product_spine_x05_authority_chain_honest() -> bool {
    crate::web_009::PRIOR_H08_RECEIPT_PATH.contains("COMPOSER_H08_2242")
        && crate::web_009::FLEET_X_MANIFEST_PATH.contains("FLEET_COMPOSER_X_100_0734")
        && crate::web_009::COMPOSER_X05_RECEIPT_PATH.contains("COMPOSER_X05_0734")
        && mcp_product_spine_h08_authority_chain_honest()
        && crate::web_009::web_009_stdio_smoke_x05_authority_chain_honest()
}

/// Build the FLEET-COMPOSER-X X05 WEB-009 stdio smoke retick probe.
#[must_use]
pub fn mcp_product_spine_x05_probe() -> McpProductSpineX05Probe {
    McpProductSpineX05Probe {
        composer_x05_job_id: crate::web_009::COMPOSER_X05_JOB_ID,
        composer_model_slug: COMPOSER_MODEL_SLUG,
        composer_x05_wave_slot: crate::web_009::COMPOSER_X05_WAVE_SLOT,
        h08: mcp_product_spine_h08_probe(),
        web_009: crate::web_009::web_009_stdio_smoke_x05_probe(),
        stdio_smoke_slot_count: crate::stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT,
        stdio_smoke_reproducible: crate::web_009::web_009_stdio_smoke_reproducible(),
        web_009_production_closed: crate::web_009::web_009_production_closed(),
        web_009_stdio_production_wired: crate::web_009::web_009_stdio_production_wired(),
        authority_chain_honest: mcp_product_spine_x05_authority_chain_honest(),
    }
}

/// X05 honesty gate — partial max; stdio GREEN without production flip invent.
#[must_use]
pub fn mcp_product_spine_x05_honest(probe: &McpProductSpineX05Probe) -> bool {
    probe.composer_x05_job_id == crate::web_009::COMPOSER_X05_JOB_ID
        && probe.composer_model_slug == COMPOSER_MODEL_SLUG
        && probe.composer_x05_wave_slot == crate::web_009::COMPOSER_X05_WAVE_SLOT
        && mcp_product_spine_h08_honest(&probe.h08)
        && crate::web_009::web_009_stdio_smoke_x05_honest(&probe.web_009)
        && probe.stdio_smoke_slot_count == crate::stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT
        && probe.stdio_smoke_reproducible
        && !probe.web_009_production_closed
        && !probe.web_009_stdio_production_wired
        && probe.authority_chain_honest
        && mcp_product_spine_x05_authority_chain_honest()
        && !probe.h08.f68.d25.d23.spine.gateway_native_wrap_closed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_spine_probe_honest_partial() {
        let probe = mcp_product_spine_probe();
        assert!(!probe.top_level_spine_present);
        assert!(probe.constitutional_tools_wired);
        assert!(!probe.promote_contribution_wired);
        assert!(!probe.refine_shape_wired);
        assert!(!probe.web_propose_delta_live_fold_wired);
        assert!(!probe.durable_contribute_queue_wired);
        assert!(!probe.gateway_native_wrap_closed);
        assert!(mcp_product_spine_honest(&probe));
    }

    #[test]
    fn mcp_spine_facet_inventory_documents_open_gaps() {
        let facets = mcp_spine_facet_inventory();
        let open: Vec<_> = facets.iter().filter(|f| !f.wired).map(|f| f.id).collect();
        assert!(open.contains(&"top_level_spine"));
        assert!(open.contains(&"promote_contribution"));
        assert!(open.contains(&"refine_shape"));
        assert!(open.contains(&"gateway_native_wrap"));
        let wired: Vec<_> = facets.iter().filter(|f| f.wired).map(|f| f.id).collect();
        assert_eq!(wired, vec!["constitutional_13_tools"]);
    }

    #[test]
    fn fleet_composer_d23_mcp_spine_metadata() {
        assert_eq!(COMPOSER_D23_JOB_ID, "FLEET-COMPOSER-D23");
        assert_eq!(COMPOSER_D23_WAVE_SLOT, "23");
        assert_eq!(COMPOSER_MODEL_SLUG, "composer-2.5");
        assert!(COMPOSER_D23_RECEIPT_PATH.contains("COMPOSER_D23_MCP_SPINE_1741"));
        assert!(mcp_product_spine_d23_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_d23_mcp_spine_honest_partial() {
        let probe = mcp_product_spine_d23_probe();
        assert!(probe.spine_honest);
        assert!(probe.authority_chain_honest);
        assert_eq!(probe.wired_facet_count, 1);
        assert_eq!(probe.open_facet_count, 6);
        assert!(mcp_product_spine_d23_honest(&probe));
    }

    #[test]
    fn fleet_composer_d25_mcp_gaps_metadata() {
        assert_eq!(COMPOSER_D25_JOB_ID, "FLEET-COMPOSER-D25");
        assert_eq!(COMPOSER_D25_WAVE_SLOT, "25");
        assert_eq!(ROUND2_MCP_GAPS_JOB_ID, "UMST-MCP-GAPS-ROUND2");
        assert!(ROUND2_GAPS_RECEIPT_PATH.contains("UMST_MCP_GAPS_ROUND2_1514"));
        assert!(COMPOSER_D25_RECEIPT_PATH.contains("COMPOSER_D25_R2_MCP_GAPS_1741"));
        assert!(mcp_product_spine_round2_absorb_honest());
    }

    #[test]
    fn fleet_composer_d25_mcp_gaps_honest_partial() {
        let probe = mcp_product_spine_d25_probe();
        assert!(probe.d23.spine_honest);
        assert!(probe.round2_absorb_honest);
        assert!(probe.d_fleet_round2_absorb_honest);
        assert_eq!(probe.d23.wired_facet_count, 1);
        assert_eq!(probe.d23.open_facet_count, 6);
        assert!(mcp_product_spine_d25_honest(&probe));
    }

    #[test]
    fn fleet_composer_f68_mcp_spine_metadata() {
        assert_eq!(COMPOSER_F68_JOB_ID, "FLEET-COMPOSER-F68-MCP-SCHEMA");
        assert_eq!(COMPOSER_F68_WAVE_SLOT, "F68");
        assert!(COMPOSER_F68_RECEIPT_PATH.contains("COMPOSER_F68_MCP_1934"));
        assert!(mcp_product_spine_f68_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_f68_mcp_spine_honest_partial() {
        let probe = mcp_product_spine_f68_probe();
        assert!(probe.gateway_schema_receipt_honest);
        assert!(probe.authority_chain_honest);
        assert_eq!(probe.wired_facet_count, 1);
        assert_eq!(probe.open_facet_count, 6);
        assert!(mcp_product_spine_f68_honest(&probe));
    }

    #[test]
    fn fleet_composer_h08_stdio_smoke_metadata() {
        assert_eq!(
            crate::stdio_smoke::COMPOSER_H08_JOB_ID,
            "FLEET-COMPOSER-H08-STDIO-SMOKE"
        );
        assert_eq!(crate::stdio_smoke::COMPOSER_H08_WAVE_SLOT, "H08");
        assert!(crate::stdio_smoke::COMPOSER_H08_RECEIPT_PATH.contains("COMPOSER_H08_2242"));
        assert!(mcp_product_spine_h08_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_h08_stdio_smoke_honest_partial() {
        let probe = mcp_product_spine_h08_probe();
        assert!(mcp_product_spine_f68_honest(&probe.f68));
        assert_eq!(probe.stdio_smoke_slot_count, 4);
        assert!(probe.stdio_smoke_reproducible);
        assert!(!probe.web_009_production_closed);
        assert!(!probe.native_stdio_smoke_production_wired);
        assert!(probe.authority_chain_honest);
        assert!(mcp_product_spine_h08_honest(&probe));
    }

    #[test]
    fn fleet_composer_x05_web_009_stdio_smoke_metadata() {
        assert_eq!(
            crate::web_009::COMPOSER_X05_JOB_ID,
            "FLEET-COMPOSER-X05-STDIO-SMOKE-RETICK"
        );
        assert_eq!(crate::web_009::COMPOSER_X05_WAVE_SLOT, "X05");
        assert!(crate::web_009::COMPOSER_X05_RECEIPT_PATH.contains("COMPOSER_X05_0734"));
        assert!(mcp_product_spine_x05_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_x05_web_009_stdio_smoke_honest_partial() {
        let probe = mcp_product_spine_x05_probe();
        assert!(mcp_product_spine_h08_honest(&probe.h08));
        assert_eq!(probe.stdio_smoke_slot_count, 4);
        assert!(probe.stdio_smoke_reproducible);
        assert!(!probe.web_009_production_closed);
        assert!(!probe.web_009_stdio_production_wired);
        assert!(probe.authority_chain_honest);
        assert!(mcp_product_spine_x05_honest(&probe));
    }
}
