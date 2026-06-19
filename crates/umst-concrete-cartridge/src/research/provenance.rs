// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Observation stamping — **immutable clock threading** (no global mutex).
//!
//! `ProvenanceClock::advance` is the sole state transition; wall time is injected via [`WallClock`] at the IO boundary.

use super::types::ObservedAt;

#[cfg(feature = "ucrs-provenance")]
use umst_ucrs::observation::UcrsObservedAt;

/// Wall-clock effect, isolated at MCP/CLI boundary.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: System time IO injection; stamp semantics on `ProvenanceClock`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WallClock;

impl WallClock {
    /// Read wall epoch milliseconds (effect at session boundary).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: `SystemTime` / UCRS wall hook; not monotonic sequence logic.
    #[must_use]
    pub fn epoch_ms(self) -> u64 {
        #[cfg(feature = "ucrs-provenance")]
        {
            return umst_ucrs::observation::wall_epoch_ms();
        }
        #[cfg(not(feature = "ucrs-provenance"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        }
    }
}

/// UCRS observation stamp mode (`UMST_UCRS_WITNESS` at session boundary).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Live vs synthetic stamp functor selection on accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UcrsStampMode {
    /// Deterministic synthetic stamps (CI default).
    #[default]
    Synthetic,
    /// Live `TemporalWitness::stamp()` path.
    Live,
}

impl UcrsStampMode {
    /// Read mode from `UMST_UCRS_WITNESS` (`live` | default synthetic).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Env IO at session boundary; stamp mode selection only.
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("UMST_UCRS_WITNESS").as_deref() {
            Ok("live") => Self::Live,
            _ => Self::Synthetic,
        }
    }
}

/// Monotonic observation sequence — threaded through accept pipeline (pure transitions).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Immutable clock state; `advance` is pure given injected wall.
#[derive(Debug)]
pub struct ProvenanceClock {
    seq: u64,
    #[cfg(feature = "ucrs-provenance")]
    mode: UcrsStampMode,
    #[cfg(feature = "ucrs-provenance")]
    live: Option<umst_ucrs::observation::TemporalWitness>,
}

impl Clone for ProvenanceClock {
    fn clone(&self) -> Self {
        Self::with_mode(self.seq, self.mode())
    }
}

impl PartialEq for ProvenanceClock {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq && self.mode() == other.mode()
    }
}

impl Eq for ProvenanceClock {}

impl Default for ProvenanceClock {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ProvenanceClock {
    /// Construct clock from environment (`UMST_UCRS_WITNESS`).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Session boundary initializer; IO on env read only.
    #[must_use]
    pub fn from_env() -> Self {
        Self::with_mode(0, UcrsStampMode::from_env())
    }

    /// Construct clock at a given UCRS sequence baseline (synthetic mode).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Clock initializer; sequence threaded through accept.
    #[must_use]
    pub fn new(seq: u64) -> Self {
        Self::with_mode(seq, UcrsStampMode::Synthetic)
    }

    /// Construct clock with explicit stamp mode.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Clock initializer with live/synthetic witness functor.
    #[must_use]
    pub fn with_mode(seq: u64, mode: UcrsStampMode) -> Self {
        #[cfg(feature = "ucrs-provenance")]
        {
            let live = if mode == UcrsStampMode::Live {
                Some(umst_ucrs::witness_for_agent(
                    &umst_ucrs::AgentConfig::default(),
                ))
            } else {
                None
            };
            Self { seq, mode, live }
        }
        #[cfg(not(feature = "ucrs-provenance"))]
        {
            let _ = mode;
            Self { seq }
        }
    }

    /// Current UCRS stamp mode.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Read-only stamp mode for session threading.
    #[must_use]
    pub const fn mode(&self) -> UcrsStampMode {
        #[cfg(feature = "ucrs-provenance")]
        {
            return self.mode;
        }
        #[cfg(not(feature = "ucrs-provenance"))]
        UcrsStampMode::Synthetic
    }

    /// Current UCRS sequence counter.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Read-only access to threaded sequence state.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.seq
    }

    /// Pure state transition: `(clock', observed_at)`.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Functional clock step; wall_ms injected at boundary only.
    #[must_use]
    pub fn advance(self, wall: WallClock) -> (Self, ObservedAt) {
        #[cfg(feature = "ucrs-provenance")]
        if self.mode == UcrsStampMode::Live {
            if let Some(mut witness) = self.live {
                let stamp = witness.stamp();
                let next_seq = stamp.ucrs_seq.unwrap_or(self.seq.saturating_add(1));
                let obs = ucrs_to_wire(&stamp, wall.epoch_ms());
                return (
                    Self {
                        seq: next_seq,
                        mode: self.mode,
                        live: Some(witness),
                    },
                    obs,
                );
            }
        }

        let next_seq = self.seq.saturating_add(1);
        let obs = observed_at_for_tick(next_seq, wall.epoch_ms());
        (
            Self {
                seq: next_seq,
                #[cfg(feature = "ucrs-provenance")]
                mode: self.mode,
                #[cfg(feature = "ucrs-provenance")]
                live: self.live,
            },
            obs,
        )
    }
}

/// Build `observed_at` for a deterministic sequence (pure given `wall_ms`).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Pure stamp from seq + wall; UCRS fields when feature enabled.
#[must_use]
pub fn observed_at_for_tick(seq: u64, wall_ms: u64) -> ObservedAt {
    #[cfg(feature = "ucrs-provenance")]
    {
        return ucrs_to_wire(&UcrsObservedAt::synthetic(seq, 0.0), wall_ms);
    }

    #[cfg(not(feature = "ucrs-provenance"))]
    {
        let _ = seq;
        ObservedAt {
            stamp_tier: "WallOnly".into(),
            ucrs_seq: None,
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(wall_ms),
        }
    }
}

/// Merge agent-supplied stamp or advance clock when Tier-2 fields are absent.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Stamp merge on accept; monotonicity checked via `is_monotonic_after`.
#[must_use]
pub fn ensure_observed_at(
    existing: Option<ObservedAt>,
    clock: ProvenanceClock,
    wall: WallClock,
) -> (ProvenanceClock, ObservedAt) {
    if let Some(obs) = existing {
        if obs.stamp_tier == "UcrsTier2" && obs.ucrs_seq.is_some() {
            return (clock, obs);
        }
        if obs.stamp_tier == "Synthetic" && obs.ucrs_seq.is_some() {
            return (clock, obs);
        }
    }
    clock.advance(wall)
}

/// Pure: candidate observation stamp is not before baseline (UCRS seq + wall + phase tie-break).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Monotonic ordering on UCRS seq with wall_ms and phase_q tie-break.
#[must_use]
pub fn is_monotonic_after(baseline: &ObservedAt, candidate: &ObservedAt) -> bool {
    let b = baseline.ucrs_seq.unwrap_or(0);
    let c = candidate.ucrs_seq.unwrap_or(0);
    if c < b {
        return false;
    }
    if c == b {
        let cw = candidate.wall_ms.unwrap_or(0);
        let bw = baseline.wall_ms.unwrap_or(0);
        if cw != bw {
            return cw >= bw;
        }
        let cp = candidate.phase_entropy_bits_q.unwrap_or(0);
        let bp = baseline.phase_entropy_bits_q.unwrap_or(0);
        return cp >= bp;
    }
    true
}

#[cfg(feature = "ucrs-provenance")]
fn ucrs_to_wire(u: &UcrsObservedAt, wall_ms: u64) -> ObservedAt {
    let obs = super::wire_v2::ucrs_observed_at_to_v2(u);
    ObservedAt {
        stamp_tier: obs.stamp_tier,
        ucrs_seq: obs.ucrs_seq,
        phase_entropy_bits_q: obs.phase_entropy_bits_q,
        phase_entropy_bits_scale: obs.phase_entropy_bits_scale,
        credit_head_bits_q: obs.credit_head_bits_q,
        credit_head_bits_scale: obs.credit_head_bits_scale,
        wall_ms: obs.wall_ms.or(Some(wall_ms)),
    }
}

/// Synthetic observed_at at current wall for gate-reject audit rows.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Convenience wrapper calling wall IO; reject stream not memory.
#[must_use]
pub fn synthetic_observed_at(seq: u64) -> ObservedAt {
    observed_at_for_tick(seq, WallClock.epoch_ms())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_monotonic_after_ucrs_seq() {
        let a = ObservedAt {
            stamp_tier: "Synthetic".into(),
            ucrs_seq: Some(1),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(100),
        };
        let b = ObservedAt {
            stamp_tier: "Synthetic".into(),
            ucrs_seq: Some(2),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(50),
        };
        assert!(is_monotonic_after(&a, &b));
        assert!(!is_monotonic_after(&b, &a));
    }

    #[test]
    fn is_monotonic_after_phase_q_when_seq_equal() {
        let a = ObservedAt {
            stamp_tier: "UcrsTier2".into(),
            ucrs_seq: Some(5),
            phase_entropy_bits_q: Some(100),
            phase_entropy_bits_scale: Some(1_000_000),
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(200),
        };
        let b = ObservedAt {
            stamp_tier: "UcrsTier2".into(),
            ucrs_seq: Some(5),
            phase_entropy_bits_q: Some(200),
            phase_entropy_bits_scale: Some(1_000_000),
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(200),
        };
        assert!(is_monotonic_after(&a, &b));
        assert!(!is_monotonic_after(&b, &a));
    }

    #[test]
    fn clock_advance_is_pure() {
        let c0 = ProvenanceClock::new(0);
        let wall = WallClock;
        let (c1, a) = c0.clone().advance(wall);
        let (c2, b) = c0.advance(wall);
        assert_eq!(c1, c2);
        assert_eq!(a.stamp_tier, b.stamp_tier);
        assert!(b.ucrs_seq.unwrap_or(0) >= a.ucrs_seq.unwrap_or(0));
    }

    #[cfg(feature = "ucrs-provenance")]
    #[test]
    fn live_advance_emits_ucrs_tier2() {
        let clock = ProvenanceClock::with_mode(0, UcrsStampMode::Live);
        let (clock, obs) = clock.advance(WallClock);
        assert_eq!(obs.stamp_tier, "UcrsTier2");
        assert!(obs.ucrs_seq.unwrap_or(0) >= 1);
        assert_eq!(clock.sequence(), obs.ucrs_seq.unwrap_or(0));
    }
}
