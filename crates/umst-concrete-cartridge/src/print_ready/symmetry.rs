// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! XY reflection symmetry on the extruded Cartesian vertex grid (topology-optimisation hook).
//!
//! The demo averages density over the four images of each node under independent reflections
//! in *x* and *y* (the Klein four group, isomorphic to **Z₂×Z₂**), not the full dihedral-8 **D₄**
//! group (which would include 90° rotations). Rotational symmetry is left to the optimiser.

use burn::tensor::{backend::Backend, Data, Int, Shape, Tensor};

/// formal_anchor: literature://symmetry-density-topology-sheet
/// formal_status: Literature
/// formal_citation: "Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055"
/// formal_form: "Index tensor `[1, N, 4]` listing the four xy-reflection partners of each primal vertex"
pub fn reflection_xy_partner_indices<B: Backend<FloatElem = f32>>(
    nx: usize,
    ny: usize,
    nz: usize,
    device: &B::Device,
) -> Tensor<B, 3, Int> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut flat: Vec<f32> = Vec::with_capacity(n * 4);
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                let nid = ix + iy * nx1 + iz * nx1 * ny1;
                let p0 = nid as f32;
                let p1 = (nx.saturating_sub(ix) + iy * nx1 + iz * nx1 * ny1) as f32;
                let p2 = (ix + ny.saturating_sub(iy) * nx1 + iz * nx1 * ny1) as f32;
                let p3 =
                    (nx.saturating_sub(ix) + ny.saturating_sub(iy) * nx1 + iz * nx1 * ny1) as f32;
                flat.extend_from_slice(&[p0, p1, p2, p3]);
            }
        }
    }
    debug_assert_eq!(flat.len(), n * 4);
    Tensor::<B, 1>::from_data(Data::new(flat, Shape::new([n * 4])), device)
        .reshape([1, n, 4])
        .int()
}

/// formal_anchor: literature://symmetry-density-topology-sheet
/// formal_status: Literature
/// formal_citation: "Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055"
/// formal_form: "Arithmetic mean of `rho` over the four xy-reflection partners (gather + mean)"
pub fn apply_reflection_xy_average<B: Backend<FloatElem = f32>>(
    rho: Tensor<B, 3>,
    partners: &Tensor<B, 3, Int>,
) -> Tensor<B, 3> {
    // `gather` (burn 0.13) requires `indices` to match `rho`'s shape on every axis except the
    // gather dimension. Partners are `[1, N, 4]`, so repeat the density channel to `[1, N, 4]`.
    let rho_wide = rho.repeat(2, 4);
    let gathered = rho_wide.gather(1, partners.clone());
    let [b, n, four] = gathered.dims();
    debug_assert_eq!(four, 4);
    let _ = b;
    gathered.mean_dim(2).reshape(Shape::new([b, n, 1]))
}
