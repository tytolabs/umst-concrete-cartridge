/* SPDX-License-Identifier: MIT
 *
 * umst_concrete_ffi.h — C header for cement chemistry FFI (cartridge fiber)
 *
 * Material-specific morphisms live here; the material-agnostic gate ABI is in umst_ffi.h.
 */

#ifndef UMST_CONCRETE_FFI_H
#define UMST_CONCRETE_FFI_H

#include <stdint.h>

#define UMST_CONCRETE_FFI_ABI_VERSION 1u

#ifdef __cplusplus
extern "C" {
#endif

uint32_t umst_concrete_ffi_abi_version(void);

float umst_hydration_degree(float age_days, float temp_c, float scm_ratio);

float umst_strength_powers(
    float wc_ratio,
    float degree_hydration,
    float air_content,
    float intrinsic_strength
);

typedef struct {
    double density;
    double free_energy;
    double hydration_degree;
    double strength;
    double max_strength;
} CThermodynamicState;

CThermodynamicState umst_thermo_state_from_mix(double w_c, double alpha, double temp);

void umst_thermo_state_from_mix_ptr(
    double w_c,
    double alpha,
    double temp,
    CThermodynamicState* out
);

#ifdef __cplusplus
}
#endif

#endif /* UMST_CONCRETE_FFI_H */
