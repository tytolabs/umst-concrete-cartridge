<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Constitutive Equations

This document specifies the canonical equations behind every constitutive
module in `src/physics/`. Each section names the source paper, the symbol
table, the form implemented, and the units. Where the implementation
deviates from the source — usually for differentiability — the deviation
is documented.

The cartridge implements `umst_manifold::core::traits::IScienceCartridge`
and exposes 22 modules grouped into six families.

---

## A. Hydration & chemistry

### A.1 `hydration` — Jennings CM-II

Source: H. M. Jennings, *Refinements to colloid model of C-S-H in cement: CM-II.*
Cem. Concr. Res. 38 (2008) 275–289.

The colloidal model partitions hydration product into low-density (LD)
and high-density (HD) C-S-H packings with degrees of hydration
$\alpha_{\mathrm{LD}}, \alpha_{\mathrm{HD}}$. The total degree of
hydration is

$$
\alpha(t) \;=\; \alpha_{\infty}
  \left[\,1 - \exp\!\left(-\left(\tfrac{t}{\tau}\right)^{\beta}\right)\,\right],
$$

with ultimate degree
$\alpha_{\infty} \approx 1.031\,(w/c) / (0.194 + w/c)$ (Mills, 1966) and
shape parameters $\tau$, $\beta$ fitted to isothermal calorimetry.

### A.2 `chemo_water` — Powers–Brownyard water binding

Source: T. C. Powers, T. L. Brownyard, *Studies of the physical properties
of hardened portland cement paste.* J. Am. Concr. Inst. 43 (1946) 101–132.

Non-evaporable water $w_n$, gel water $w_g$, and capillary water $w_c$
satisfy

$$
w_n \;=\; 0.227 \,\alpha\, c, \qquad
w_g \;=\; 0.190 \,\alpha\, c, \qquad
w_c \;=\; w - (w_n + w_g),
$$

with $w$ the initial mix water mass and $c$ the cement mass.

### A.3 `set_time` — Wadsö isothermal heat-flux setting

Source: L. Wadsö, *Operational issues in isothermal calorimetry.*
Cem. Concr. Res. 40 (2010) 1129–1137; ASTM C191 cross-validation.

Initial set is defined when cumulative heat
$Q(t) \ge 50\ \mathrm{J/g_{cement}}$;
final set when $Q(t) \ge 250\ \mathrm{J/g_{cement}}$ at 20 °C.

### A.4 `thermo` — Schindler–Folliard heat of hydration

Source: A. K. Schindler, K. J. Folliard, *Heat of hydration models for
cementitious materials.* ACI Mater. J. 102 (2005) 24–33.

$$
H(t) = H_u\, \alpha(t), \qquad
H_u = 500\,p_{C_3S} + 260\,p_{C_2S} + 866\,p_{C_3A} + 420\,p_{C_4AF}
       + 624\,p_{SO_3} + 1186\,p_{FreeCa} + 850\,p_{MgO}
\ \mathrm{[J/g]},
$$

with $p_x$ the mass fraction of the Bogue phase $x$.

---

## B. Microstructure

### B.1 `nano` — Pellenq C-S-H molecular density

Source: R. J.-M. Pellenq et al., *A realistic molecular model of cement
hydrates.* PNAS 106 (2009) 16102–16107.

Effective C-S-H density
$\rho_{\mathrm{C\text{-}S\text{-}H}}(\mathrm{Ca/Si}, w/c)$ is fit to the
PNAS calibration grid; gradients flow through bilinear interpolation.

### B.2 `colloidal` — DLVO interaction

Source: B. V. Derjaguin, L. D. Landau, E. J. W. Verwey, J. T. G. Overbeek;
modern review in R. J. Flatt, P. Bowen, *Yodel: a yield stress model for
suspensions.* J. Am. Ceram. Soc. 89 (2006) 1244–1256.

Pair interaction
$U(h) = U_{\mathrm{vdW}}(h) + U_{\mathrm{el}}(h)$
with attractive Hamaker term
$U_{\mathrm{vdW}} = -A_H\, R / (12 h)$ and screened-electrostatic
$U_{\mathrm{el}} = 2\pi \varepsilon \varepsilon_0 R \zeta^2
\ln\!\left[1 + \exp(-\kappa h)\right]$.
$A_H$ is the Hamaker constant, $\zeta$ the zeta potential,
$\kappa^{-1}$ the Debye length.

### B.3 `porosity` — Powers–Brownyard porosity

$$
\phi_{\mathrm{cap}} \;=\;
   \frac{w/c - 0.36\,\alpha}{w/c + 0.32}, \qquad
\phi_{\mathrm{gel}} \;=\; 0.28\,\alpha.
$$

### B.4 `itz` — Scrivener interfacial transition zone

Source: K. L. Scrivener et al., *The interfacial transition zone (ITZ)
between cement paste and aggregate.* Interface Sci. 12 (2004) 411–421.

ITZ porosity decays exponentially from the aggregate surface with a
characteristic length $\ell_{\mathrm{ITZ}} \sim 20$–$50\ \mu\mathrm{m}$.

### B.5 `packing` — modified Andreasen–Andersen

$$
P(D) \;=\; \frac{D^{q} - D_{\min}^{q}}{D_{\max}^{q} - D_{\min}^{q}},
\qquad q \in [0.30, 0.45].
$$

---

## C. Rheology & printing

### C.1 `rheology` — Chateau–Ovarlez–Trung / YODEL

Sources: X. Chateau, G. Ovarlez, K. L. Trung, *Homogenization approach to
the behavior of suspensions of noncolloidal particles in yield stress fluids.*
J. Rheol. 52 (2008) 489–506; Flatt & Bowen YODEL (above).

Yield stress and plastic viscosity in a paste–aggregate suspension:

$$
\tau_y(\phi) = \tau_y^{(0)}\, \sqrt{\,(1-\phi)\,(1-\phi/\phi_m)^{-2.5\,\phi_m}}, \qquad
\eta_p(\phi) = \eta_0\,(1-\phi/\phi_m)^{-2.5\,\phi_m},
$$

with paste yield $\tau_y^{(0)}$, aggregate volume fraction $\phi$, and
maximum packing $\phi_m$.

### C.2 `printability` — Roussel buildability and extrudability

Source: N. Roussel, *Rheological requirements for printable concretes.*
Cem. Concr. Res. 112 (2018) 76–85.

Buildability — failure under self-weight at print height $H$:

$$
\tau_y \;\ge\; \frac{\rho g H}{\sqrt{3}}.
$$

Extrudability — pumping pressure
$\Delta P = (2 L / R)\,\tau_y + (8 L Q / \pi R^4)\,\eta_p$
with nozzle radius $R$, length $L$, and flow rate $Q$.

---

## D. Mechanics

### D.1 `strength` — Jennings CM-II compressive strength

Source: Jennings 2008 (above), and H. M. Jennings, J. J. Thomas et al.,
*A multi-technique investigation of the nanoporosity of cement paste.*
Cem. Concr. Res. 37 (2007) 329–336.

$f_c(t) \propto \alpha(t)^a \,\rho_{\mathrm{C\text{-}S\text{-}H}}^{\,b}\,
                (1 - \phi_{\mathrm{cap}})^{c}$
with $(a, b, c)$ fitted to a CM-II calibration set.

### D.2 `fracture` — Ulm–Coussy micromechanics

Source: F.-J. Ulm, O. Coussy, *Mechanics and Durability of Solids: Volume 1*,
MIT Press 2003; F.-J. Ulm, *Strength scaling of cementitious materials.*
J. Eng. Mech. 133 (2007).

Fracture toughness via porosity-corrected effective modulus
$E_{\mathrm{eff}} = E_0 (1 - \phi)^n$ and
$K_{Ic} = \sqrt{2 \gamma_s E_{\mathrm{eff}}}$ with
$\gamma_s$ the surface energy.

### D.3 `creep` — Bažant B4

Source: Z. P. Bažant, M. Jirásek et al., *RILEM model B4 for creep, drying
shrinkage and autogenous shrinkage of normal and high-strength concretes.*
Mater. Struct. 48 (2015) 753–770.

Compliance $J(t, t')$ as a sum of basic, drying, and autogenous components
with parameters fitted to the RILEM database.

### D.4 `shrinkage` — Bažant–Baweja

$$
\varepsilon_{\mathrm{sh}}(t) = \varepsilon_{\mathrm{sh}}^{\infty}
  \tanh\!\left(\sqrt{(t - t_0)/\tau_{\mathrm{sh}}}\right).
$$

### D.5 `fiber` — Naaman pullout micromechanics

Source: A. E. Naaman, *Fiber-reinforced concrete: from material to product.*
ACI SP-235 (2006). Critical fibre volume fraction
$V_{f,\mathrm{crit}} = \sigma_{\mathrm{cu}} /
(\eta_l \eta_o\, \tau_b\, l_f / d_f)$.

### D.6 `polymer` — Su–Bijen latex modification

Source: Z. Su, K. van Breugel, J. Bijen, *Influence of polymer modification
on the hydration of portland cement.* Cem. Concr. Res. 21 (1991).
Polymer film reduces effective porosity by $\Delta\phi = -k_p\, p$ with
$p$ the polymer mass fraction.

---

## E. Durability

### E.1 `freeze_thaw` — Powers spacing factor

Source: T. C. Powers, *The air requirement of frost-resistant concrete.*
Highw. Res. Board Proc. 29 (1949) 184–211. Critical spacing factor
$\bar{L} \le 0.20\ \mathrm{mm}$ for ASTM C666 durability.

### E.2 `transport` — Tang–Nilsson chloride diffusivity

Source: L. Tang, L. O. Nilsson, *Rapid determination of the chloride
diffusivity in concrete by applying an electrical field.* ACI Mater. J.
89 (1992) 49–53.

$$
D_{\mathrm{Cl}} = \frac{R T L}{z F U}\,\frac{x_d}{t_d},
$$

with non-steady-state migration depth $x_d$ at time $t_d$.

### E.3 `self_heal` — Edvardsen autogenous healing

Source: C. Edvardsen, *Water permeability and autogenous healing of cracks
in concrete.* ACI Mater. J. 96 (1999) 448–454. Recovery follows a
$\sqrt{t}$ law in crack water flux.

---

## F. Sustainability and economics

### F.1 `sustainability` — EN 15804 GWP

$$
\mathrm{GWP}_{\mathrm{mix}}
  = \sum_i m_i\, e_i\quad
  \text{[kg CO}_2\text{e / m}^3\text{]},
$$

with $e_i$ the embodied $\mathrm{CO_2}$-equivalent of constituent $i$
from EPDs aligned to EN 15804+A2.

### F.2 `cost` — multi-objective auxiliary

$\mathrm{Cost}_{\mathrm{mix}} = \sum_i m_i\, c_i$ with $c_i$ the unit
cost in the user-supplied price book. Used as an auxiliary objective
during gradient-based mix optimisation.

---

## G. Differentiability and units

Every module above is implemented as a pure function of `burn` tensors,
so reverse-mode autodiff propagates gradients of any output (e.g.
28-day compressive strength, slump-flow yield, embodied $\mathrm{CO_2}$)
with respect to any mix-design scalar, including superplasticiser
dosage. Where a published formula contains a $\min$, $\max$, or piecewise
branch, it is replaced with a smooth approximation
($\mathrm{softplus}$, $\tanh$ blending) and the smoothing parameter is
documented in the corresponding module.

All public fields and tensors carry SI units in the rustdoc. A
quick reference:

| Quantity | Unit |
|----------|------|
| stress, pressure | Pa |
| viscosity | Pa·s |
| temperature | K |
| time | s |
| length | m |
| mass per volume | kg / m³ |
| heat of hydration | J / g$_{\text{cement}}$ |
| diffusivity | m² / s |

When a legacy non-SI unit appears in a published formula (e.g. MPa for
compressive strength), the conversion is performed at the module
boundary and the public API always exposes SI.

---

## H. References

The references above are repeated here for convenience, with stable
identifiers where available.

- Bažant, Z. P., Jirásek, M., Hubler, M. H., Carol, I. (2015). RILEM
  model B4. *Mater. Struct.* 48, 753–770. doi:10.1617/s11527-014-0485-2.
- Chateau, X., Ovarlez, G., Trung, K. L. (2008). *J. Rheol.* 52,
  489–506. doi:10.1122/1.2838254.
- Edvardsen, C. (1999). *ACI Mater. J.* 96, 448–454.
- Flatt, R. J., Bowen, P. (2006). YODEL. *J. Am. Ceram. Soc.* 89,
  1244–1256. doi:10.1111/j.1551-2916.2005.00888.x.
- Jennings, H. M. (2008). CM-II. *Cem. Concr. Res.* 38, 275–289.
  doi:10.1016/j.cemconres.2007.10.006.
- Naaman, A. E. (2006). ACI SP-235.
- Pellenq, R. J.-M. et al. (2009). *PNAS* 106, 16102–16107.
  doi:10.1073/pnas.0902180106.
- Powers, T. C. (1949). *Highw. Res. Board Proc.* 29, 184–211.
- Powers, T. C., Brownyard, T. L. (1946). *J. Am. Concr. Inst.* 43,
  101–132.
- Roussel, N. (2018). *Cem. Concr. Res.* 112, 76–85.
  doi:10.1016/j.cemconres.2018.05.008.
- Schindler, A. K., Folliard, K. J. (2005). *ACI Mater. J.* 102, 24–33.
- Scrivener, K. L. et al. (2004). *Interface Sci.* 12, 411–421.
  doi:10.1023/B:INTS.0000042339.92990.4c.
- Tang, L., Nilsson, L. O. (1992). *ACI Mater. J.* 89, 49–53.
- Ulm, F.-J., Coussy, O. (2003). *Mechanics and Durability of Solids,
  Vol. 1.* MIT Press.
- Wadsö, L. (2010). *Cem. Concr. Res.* 40, 1129–1137.
  doi:10.1016/j.cemconres.2010.06.011.
