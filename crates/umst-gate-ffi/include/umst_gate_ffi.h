/* SPDX-License-Identifier: MIT */
/* Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO */

/*
 * umst_gate_ffi.h — C-ABI thermodynamic gate (Layer 0).
 * Chemistry scalars: umst_concrete_ffi.h (do not conflate).
 */

#ifndef UMST_GATE_FFI_H
#define UMST_GATE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define UMST_GATE_FFI_ABI_VERSION 1u

typedef struct {
    uint8_t admissible; /* 1 = PASS admissible, 0 = reject */
    int32_t verdict;    /* 0 = PASS, 1 = REJECT, 2 = WARN */
} CGateSummary;

uint32_t umst_gate_ffi_abi_version(void);

/*
 * Returns 0 on success.
 * Negative: -1 null/utf8, -2 mix JSON parse, -3 profile/gate, -4 out_json too small.
 * out_json may be NULL to skip JSON copy. When non-NULL, receives NUL-terminated
 * canonical (sorted-key) GateCheckResult JSON.
 */
int umst_gate_check(
    const char* profile_id,
    const char* mix_json,
    int explain,
    uint64_t ucrs_seq,
    uint64_t wall_ms,
    CGateSummary* out_summary,
    char* out_json,
    size_t out_json_len);

#ifdef __cplusplus
}
#endif

#endif /* UMST_GATE_FFI_H */
