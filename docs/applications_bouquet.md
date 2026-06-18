# The UMST Application Bouquet: Scientific & Industrial Architecture

The Universal Material State Tensor (UMST) is a massive thermodynamic engine written in high-performance Rust (`burn::tensor`). While it is mathematically capable of predicting everything from a simple driveway mix to an alien Martian habitat, accessing that power requires understanding **how** the science works, **what physical tests** are required to calibrate it, and the specific **limits** of the current tensor graph.

---

## 1. The Compute & Hardware Framework (Landauer Reality)
UMST natively calculates the thermodynamic energy required to solve its own PDE grids. Because it uses collapsed-batch tensor semantics ("computing everything, everywhere, all at once"), we can cleanly categorize applications by their computational cost:

*   **Nano-Compute (1–10 ms):** Single-voxel thermodynamics (e.g., predicting the strength of 1 mix). *Hardware:* Runs instantly on any standard laptop or processor.
*   **Micro-Compute (2–10 seconds):** Batch data operations (e.g., auditing 10,000 historical Ready-Mix designs). *Hardware:* Standard commercial laptop (no GPU required).
*   **Macro-Compute (Minutes to Hours):** Multi-voxel grid PDE solves and spatial simulations. *Examples:* Predicting 3D thermal gradients across a 20-meter dam wall; simulating a full 3DCP robotic toolpath over thousands of printed layers to check for buckling. *Hardware:* Modern MacBook Pro (M-Series) or dedicated Desktop.
*   **Meta-Compute (Hours to Days):** Reinforcement Learning (PPO) geometric generation and topology optimization. *Hardware:* Apple Silicon M-Max/Ultra, Nvidia CUDA grids, or Cloud Clusters.

## 2. The Ecosystem UI Tools
Depending on the application, users access the UMST Manifold via the specific interface closures offered in our Git repository:
*   **Command Line Interface (`umst-cli`):** High-speed batch processing for data-engineers and concrete plants.
*   **Python SDK (`umst-py`):** Deep integration for PhD researchers, data scientists, and ML pipelines.
*   **Rhino / Grasshopper (`umst-gh`):** Visual 3D node-based scripting for architects and roboticists.

---

## 3. The MI / Epistemic Selector System (Multimodal Agent Routing)
When an autonomous agent (such as a policy orchestrator or a PPO Gladiator) runs the UMST manifold in the wild, it rarely receives clean, perfectly structured inputs. It receives **multimodal data**: images of cracked concrete, PDF site reports, unstructured text observations, and historical CSVs. 

To handle this safely without crashing the rigorous thermodynamic PDE grids, the agent employs an **MI (Material Intelligence) / Epistemic Selector System**. This cognitive triage layer assesses the *epistemic certainty* of the incoming multimodal data before deciding how to use it:
*   **High Certainty (Structured Physics):** Structured datasets with exact chemical composition (e.g., specific oxide percentages from XRF, mass-loss data) are routed directly into the `burn::tensor` graph for hard physics calculations.
*   **Medium Certainty (Empirical History):** A database of past 28-day break tests is routed to the TOML Gradient Descent loop to calibrate $s_{intrinsic}$.
*   **Low Certainty (Unstructured/Multimodal):** A photograph of a buckling 3D-printed wall is passed through a Vision-Language Model (VLM). The Selector refuses to map this directly to a hard physics tensor. Instead, it extracts the buckling boolean (True/False) and routes it to the **Tier 4 RL Residual Bridge** to penalize the PPO geometry policy.

This Epistemic Selector ensures that messy, multimodal "real-world" intelligence safely interfaces with the strict mathematical bounds of UMST.

---

## 4. Development Directions: The Four Operational Tiers
How does the system handle missing data or extreme physical limits?
1.  **Tier 1 (Pure Physics):** Zero historical data. Solves entirely via first-principles math.
2.  **Tier 2 (Dataset Calibration):** Historical plant data exists. The engine uses TOML profiles to tune internal constants (e.g. `s_intrinsic`) to match empirical reality.
3.  **Tier 3 (PPO AI Optimization):** Wraps the physics engine in a Reinforcement Learning loop to generate novel designs.
4.  **Tier 4 (RL Residual Bridge):** When operating at extreme boundaries where a formal tensor closure does not yet exist, but a physical dataset *does*, the RL system learns the delta between the baseline physics and empirical reality.

---

# Example Application Domains

> [!IMPORTANT]
> **Crucial Architecture Note:** The `orchestrator.rs` DAG evaluates physics in a **single deterministic forward pass**. The outputs of early closures (like hydration $\alpha$) cascade into subsequent tensors (porosity, strength). While all general physics (thermodynamics, mechanics, transport) execute sequentially for every mix, specialty closures like `polymer.rs` and `fiber.rs` dynamically activate only when their respective material input fractions are non-zero.
> 
> The **"Key Highlighted Tensors"** listed below simply surface the specific mathematical closures that drive the focal point of the application workflow.

## I. High-Volume Construction

### 00. Residential & Hand-Mixed Concrete (DIY)
*   **Application:** Multi-objective optimization balancing minimum required compressive strength against hyper-local ambient weather constraints, ensuring the mix does not fail due to rapid desiccation or freezing, while maintaining a workable slump for manual placement.
*   **Key Highlighted Tensors:** Focuses heavily on the `hydration.rs` tensor graph. The Arrhenius kinetics scale the reaction based on an assumed reference temperature ($T_{ref} = 293.15K$).
*   **Calibration Workflow:** The orchestrator defaults to the bundled `default.v1.toml` profile. The user inputs basic bagged cement properties ($w/c$ ratio, local temperature) without needing advanced laboratory tests. The system uses a generic $s_{intrinsic}$ derived from historical OPC datasets to provide a safe, conservative strength estimate.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute (Instant).

### 0. Ready-Mix Concrete (RMC) & Commercial Batching
*   **Application:** Pareto-front optimization of thousands of historical mix designs, simultaneously minimizing economic unit cost ($/m³) and Global Warming Potential (GWP), while strictly satisfying non-negotiable 28-day structural strength limits and avoiding autogenous shrinkage cracking.
*   **Key Highlighted Tensors:** Extensively utilizes `cost.rs` (a pure linear dot-product tensor), `strength.rs`, `shrinkage.rs` (Bažant B4 autogenous microstrain), and `hydration.rs`.
*   **Calibration Workflow:** The orchestrator uses the `uci_d1` profile by default. For higher precision, the plant manager extracts a CSV of their last 1,000 historical 28-day break tests. They run `umst-cli calibrate`. The engine performs gradient descent to find the exact $s_{intrinsic}$, $k_{slag}$, and $k_{fly\_ash}$ constants that minimize the Mean Absolute Error (MAE) for their specific materials, generating a custom `.toml` profile for future predictions.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Micro-Compute (Seconds for 10,000 mixes).

### 1. Precast Concrete Elements
*   **Application:** Optimizing the factory throughput cycle by balancing the thermal energy cost of steam-curing against the structural requirement for rapid early-age strength, while minimizing long-term viscoelastic creep deflection in the final installed element.
*   **Key Highlighted Tensors:** Relies on `thermo.rs` (accelerated kinetics), `creep.rs` (Bažant RILEM B4 viscoelastic compliance including basic and drying creep), and `clinker_eos.rs` (Vinet Equation of State determining the bulk modulus of the hydrated phases).
*   **Calibration Workflow:** The orchestrator assumes a default activation energy $E_a = 40,000$ J/mol and an ambient relative humidity $RH = 0.55$. To track exact factory steaming, the user maps the time-temperature profile into the inputs. For exact creep compliance, they define the exact loading age ($t_{load}$).
*   **Recommended UI:** Command Line Interface (`umst-cli`) or Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute (Instant).

## II. Advanced Manufacturing & Automation

### 2. 3D Concrete Printing (3DCP)
*   **Application:** Dynamic multi-objective optimization balancing the competing requirements of extrudability (low viscosity to prevent pump blockage) and buildability (rapid structural buildup to prevent gravitational buckling), while simultaneously minimizing the total cement binder to reduce carbon footprint.
*   **Key Highlighted Tensors:** Gated by `printability.rs`. Evaluates Roussel's Buildability Model ($h_{crit} = \frac{\tau_{effective}}{\rho g}$) and extrudability constraints.
*   **Calibration Workflow:** The orchestrator initializes with safety defaults: Static Yield Stress $\tau_0 = 120.0$ Pa, Plastic Viscosity $= 45.0$ Pa·s, and characteristic time $t_{char} = 120.0s$. To map exactly to a specific robotic pump, the user runs a Rheometer Sweep to measure their material's true $\tau_0$ and dynamic thixotropic structural buildup rate ($A_{thix}$), injecting these scalars into the Python solver.
*   **Recommended UI:** Rhino / Grasshopper (`umst-gh`).
*   **Compute Level:** Macro-Compute (Minutes for full toolpath simulation).

### 3. PPO Geometric Generation (Smart Gladiator)
*   **Application:** Inverse-design topology optimization where AI agents hallucinate millions of novel geometric structures, aggressively minimizing volumetric mass and economic cost while strictly evading physical buckling or thermal fracturing gates.
*   **Key Highlighted Tensors:** PPO agent acts on geometry policies, while UMST executes the entire `orchestrator.rs` pass. `cost.rs` and `sustainability.rs` act as continuous reward gradients.
*   **Calibration Workflow:** The user defines the bounding box limits, the action space (e.g., nozzle translation vectors), and the Reward Function directly in Python. Calibration here involves tuning the PPO hyperparameters (learning rate, entropy coefficient) and determining the exact penalty weights (e.g., a buckling failure = -100 reward, while excess embodied carbon = -1 per kg).
*   **Recommended UI:** Python SDK (`umst-py` via `ray[rllib]`).
*   **Compute Level:** Meta-Compute (Hours to Days on GPU).

### 4. Shotcrete (Sprayed Concrete)
*   **Application:** Optimizing the delicate balance between pumpability (requires fluidity) and instantaneous adhesion (requires massive static yield stress to prevent rebound/fall-off), while managing the extreme long-term strength penalties caused by alkaline set-accelerators.
*   **Key Highlighted Tensors:** Leverages `rheology.rs` to compute instantaneous static yield stress ($\tau_0$) via YODEL mechanics, and `set_time.rs` to calculate accelerated chemical affinity.
*   **Calibration Workflow:** The user runs an Early-Age Penetrometer Test to calibrate the dosage of alkaline set-accelerators, mapping this to the `early_boost` parameter in their chosen TOML schema. This allows the engine to accurately model flash-setting behavior.
*   **Recommended UI:** Python SDK (`umst-py`) or Command Line Interface (`umst-cli`).
*   **Compute Level:** Micro-Compute.

## III. Extreme Environments & Infrastructure

### 5. Mass Concrete (Dams & Foundations)
*   **Application:** Optimizing the thermodynamic balance between minimizing the massive internal adiabatic heat spike (to prevent thermal fracturing) and maintaining sufficient early-age strength for construction schedules, typically by maximizing SCM replacement ratios.
*   **Key Highlighted Tensors:** `thermo.rs` computes internal heat generation, coupled tightly with `creep.rs` to assess viscoelastic stress relaxation over the cooling period.
*   **Calibration Workflow:** The orchestrator passes a baseline Activation Energy ($E_a = 40,000$ J/mol) and tracks an adiabatic temperature rise proxy. For massive pours, the engineer performs an Adiabatic Calorimetry Test to extract the precise heat evolution curve and modifies the default scalars. They also map the thermal conductivity of the specific aggregate into the spatial grid.
*   **Recommended UI:** Python SDK (`umst-py` for spatial thermal grids) or Rhino / Grasshopper (`umst-gh`).
*   **Compute Level:** Macro-Compute (Minutes).

### 6. Marine & Subsea Infrastructure
*   **Application:** Multi-objective lifecycle design maximizing capillary pore refinement (to block rapid chloride and sulfate ingress) while balancing the slower hydration kinetics of high-slag cement blends in cold ocean environments.
*   **Key Highlighted Tensors:** `transport.rs` coupled with `porosity.rs`. Computes the capillary diffusion coefficient ($D_{cl}$) based on gel-space ratios and SCM densification.
*   **Calibration Workflow:** The orchestrator anchors the diffusion logic with a reference diffusivity factor ($D_{ref} = 10^{-12} \text{ m}^2/\text{s}$). The user performs an NT BUILD 492 test to fine-tune this scalar for their specific blend, and switches to the bundled `highscm` profile to accurately model the pore-refinement effect of high slag/fly ash substitutions.
*   **Recommended UI:** Command Line Interface (`umst-cli`) or Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

### 7. Cold Weather Concreting
*   **Application:** Balancing the need for rapid early-age hydration (to resist freezing) with the requirement for an optimized entrained air-void network, ensuring the capillary pores can absorb catastrophic ice-lens expansion without structural fracturing.
*   **Key Highlighted Tensors:** Calculates sub-zero hydration stalling via `hydration.rs` Arrhenius exponential decay, and evaluates internal fracture stress via `freeze_thaw.rs`.
*   **Calibration Workflow:** The orchestrator assumes a conservative air content ($4\%$), spacing factor ($35.0$ mm), and simulates 6 freeze-thaw cycles. If the local mix design has specific entrained air metrics, the user overrides these defaults via an ASTM C457 Microscopic Air-Void Analysis dataset.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 8. Lunar & Martian Construction (ISRU)
*   **Application:** Multi-objective structural optimization under extreme resource scarcity, balancing the mechanical strength of in-situ regolith against the high energy costs of synthesizing cementitious precursors in low-gravity, vacuum environments.
*   **Key Highlighted Tensors:** Core thermodynamics and `printability.rs`, with overriding body force scalars and vacuum moisture transport models (`chemo_water.rs`).
*   **Calibration Workflow:** Because standard terrestrial constants fail, the user generates a specific TOML profile based on chemical spectrometry data for Lunar Mare basalt or Martian regolith simulant. They manually override the gravitational body force scalar (e.g., $g=1.62$ for Lunar) in the Python solver constraints.
*   **Recommended UI:** Python SDK (`umst-py`) or Rhino / Grasshopper (`umst-gh`).
*   **Compute Level:** Nano-Compute.

### 9. Deep Oil Well Cementing
*   **Application:** Precision tuning of extreme retardation agents to ensure the slurry remains perfectly liquid during high-pressure pumping miles underground, while guaranteeing an instantaneous set the moment it reaches the target depth (the "right-angle set").
*   **Key Highlighted Tensors:** Tracks massive structural buildup and retardation via `rheology.rs` and `set_time.rs` modified by extreme temperatures.
*   **Calibration Workflow:** Because standard hydration constants break down at 150°C and 10,000 psi, the user runs an API Consistometer Test (thickening time) under HPHT conditions and generates a specialized TOML profile bound specifically to that high-temperature dataset.
*   **Recommended UI:** Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

## IV. Next-Generation & Sustainable Materials

### 10. Ultra-High Performance Concrete (UHPC)
*   **Application:** Extreme multi-objective optimization balancing maximum nano-particle packing density (for >150 MPa strength) against severe autogenous shrinkage risks, while perfectly spacing micro-steel fibers to bridge brittle fractures.
*   **Key Highlighted Tensors:** Gated heavily by `nano.rs`, `packing.rs`, and conditionally activates `fiber.rs` if micro-steel fibers are present. Extremely sensitive to `shrinkage.rs` limits due to near-zero free water.
*   **Calibration Workflow:** The researcher selects the bundled `uhpc` profile. They perform Laser Diffraction Particle Size Analysis on their silica fume, quartz flour, and cement, inputting the exact $D_{10}$, $D_{50}$, and $D_{90}$ distributions into the `packing.rs` model. They map the exact volume fraction and tensile yield strength of the steel fibers into the active tensor block.
*   **Recommended UI:** Command Line Interface (`umst-cli`) or Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

### 11. Geopolymers & Alkali-Activated Materials
*   **Application:** Total environmental optimization, swapping the massive CO2 footprint of Portland Cement for an alkali-activated industrial waste stream, while balancing the aggressive chemical molarity required for polymerization against worker safety and cost.
*   **Key Highlighted Tensors:** Conditionally activates `polymer.rs` when precursor fractions are high. Evaluates raw Global Warming Potential natively via `sustainability.rs`.
*   **Calibration Workflow:** The chemist determines the precise amorphous silica and alumina content via X-Ray Fluorescence (XRF) and defines the molarity of the activator solution. Setting the cement fraction to zero dynamically triggers the `polymer.rs` tensor pathway in the orchestrator.
*   **Recommended UI:** Python SDK (`umst-py`) or Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 12. Carbon-Negative / CO2 Injected Concrete
*   **Application:** Optimizing the dynamic capture and sequestration of supercritical CO2 into wet concrete, maximizing carbon mineralization (limestone formation) while utilizing the resulting kinetic heat spike to accelerate construction timelines.
*   **Key Highlighted Tensors:** Directly tracks embodied carbon metrics via `sustainability.rs` while analyzing the kinetic acceleration of early-age hydration.
*   **Calibration Workflow:** The user injects the exact dose of $CO_2$ (in grams per kg of cement) and utilizes Calorimetry data to map the accelerated heat spike caused by nano-calcium carbonate seeds. This fine-tunes the kinetics multiplier ($k_{ref}$) in the TOML profile.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 13. Self-Healing Concrete
*   **Application:** Lifecycle durability optimization balancing the upfront economic cost of embedding encapsulated healing agents (bacteria or silicates) against the long-term maintenance savings achieved by autogenous crack sealing.
*   **Key Highlighted Tensors:** Handled directly by `self_heal.rs`. Calculates healing potential based on internal relative humidity and unhydrated cement fractions ($\alpha$).
*   **Calibration Workflow:** The orchestrator sets an internal humidity default of $RH_{internal} = 0.92$ and links to the bundled `selfheal` profile. The user runs a Water Permeability/Crack Healing Test to measure flow stoppage over 28 days, extracting an empirical "Healing Rate Constant" to map precisely to the target tensor.
*   **Recommended UI:** Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

### 14. Recycled Aggregate Concrete (RAC)
*   **Application:** Circular economy optimization balancing the environmental benefits of using crushed demolition waste against the structural penalties of weaker Interfacial Transition Zones (ITZ) and highly variable water absorption rates.
*   **Key Highlighted Tensors:** Evaluated via `itz.rs`. Calculates the Interfacial Transition Zone (ITZ) porosity and thickness to penalize aggregate-paste bond strength.
*   **Calibration Workflow:** The orchestrator assumes standard natural aggregate porosity. Because crushed brick absorbs massive amounts of water, the user performs a 24-hour Water Absorption Test and maps the 5-10% capacity into the `porosity.rs` inputs to stop the engine from artificially stalling the hydration reaction.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 15. Smart / Piezoresistive Concrete
*   **Application:** Optimizing the spatial distribution of carbon nanotubes to achieve electrical percolation and piezoresistivity (self-sensing stress detection), without severely compromising the rheological workability or economic cost of the mix.
*   **Key Highlighted Tensors:** Evaluates the volume fraction of conductive fillers via the `optical.rs` tensor (which computes generalized electromagnetic/optical transport properties including piezoresistive thresholds).
*   **Calibration Workflow:** The researcher runs an Electrical Impedance Spectroscopy (EIS) test to measure the bulk conductivity of the matrix at various nanomaterial dosages. They use this dataset to map the exact Percolation Threshold into the Python `umst-py` graph.
*   **Recommended UI:** Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

## V. Specialized Functional Concrete

### 16. Underwater Anti-Washout Concrete
*   **Application:** Balancing the competing need for high fluidity (to allow the concrete to self-level in trenches) against the requirement for massive colloidal stickiness to prevent the cement paste from washing away into the surrounding river currents.
*   **Key Highlighted Tensors:** Directly engages `colloidal.rs` to compute the DLVO interaction potentials (nanoscale stickiness) and limits.
*   **Calibration Workflow:** The orchestrator initializes with defaults: Zeta Potential $\zeta = -25.0$ mV, ionic strength $0.03$, and separation distance $50.0$ nm. The user conducts a Washout Mass Loss Test to calibrate the exact dosage of Anti-Washout Admixtures, mapping the viscosity modifiers into the $\eta_p$ parameter of the Bingham `rheology.rs` closure.
*   **Recommended UI:** Python SDK (`umst-py`).
*   **Compute Level:** Nano-Compute.

### 17. Nuclear Shielding Concrete
*   **Application:** High-stakes multi-objective optimization maximizing the absolute density of the matrix (for gamma ray shielding) and total bound water content (for neutron moderation), while ensuring the structural lattice survives decades of localized radiation exposure.
*   **Key Highlighted Tensors:** Tracks specific aggregate densities and evaluates absolute bound-water content via `porosity.rs`, while `clinker_eos.rs` maintains the high-pressure bulk moduli references.
*   **Calibration Workflow:** The nuclear engineer defines the exact Specific Gravity (SG) of the heavy aggregates (e.g., Magnetite SG = 5.1). They run Thermogravimetric Analysis (TGA) to measure non-evaporable water ($W_n$) and inject this mass fraction into the `porosity.rs` constraints to ensure long-term shielding viability.
*   **Recommended UI:** Python SDK (`umst-py`) or Rhino / Grasshopper (`umst-gh`).
*   **Compute Level:** Nano-Compute.

### 18. Pervious Concrete (Drainage Pavements)
*   **Application:** Stormwater optimization balancing the necessity of a massive, highly interconnected void network (to drain flash floods) against the catastrophic structural weakness caused by removing the fine aggregate matrix.
*   **Key Highlighted Tensors:** Disables standard capillary diffusion logic and evaluates macroscopic `packing.rs` void generation based on extreme gap-graded aggregate fractions.
*   **Calibration Workflow:** The civil engineer runs a Falling Head Permeability Test on cylindrical cores to extract the exact hydraulic conductivity coefficient. They map this coefficient, along with the total void ratio, into the `packing.rs` and `porosity.rs` tensors.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 19. Autoclaved Aerated Concrete (AAC)
*   **Application:** Thermodynamic optimization maximizing the generation of hydrogen gas bubbles (to dramatically lower bulk thermal conductivity for building insulation) without causing the plastic matrix to structurally collapse before the autoclave curing phase.
*   **Key Highlighted Tensors:** Evaluates bulk thermal conductivity via `thermo.rs` and calculates massive macroscopic void expansion ratios.
*   **Calibration Workflow:** The user runs a Volume Expansion Test during the early plastic phase to measure the height rise of the foam, inputting the Expansion Rate constant into the Python orchestrator. They measure the hardened Thermal Conductivity via a Hot Disk test to validate the insulation performance.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.

### 20. Soil Stabilization / Rammed Earth
*   **Application:** Optimizing extreme low-binder matrices, balancing the desire for zero-carbon, locally sourced earth construction against the risk of catastrophic capillary water absorption and structural liquefaction during heavy rain.
*   **Key Highlighted Tensors:** Evaluates very early age strength and extremely low binder fractions using `strength.rs`, while monitoring moisture limits via `chemo_water.rs`.
*   **Calibration Workflow:** The geotechnical engineer runs a Proctor Compaction Test to determine the Optimum Moisture Content and Maximum Dry Density of their local soil, mapping these exact scalars into the `packing.rs` profile. They run Atterberg Limits to map clay plasticity, adapting standard OPC mechanics to a highly cohesive matrix.
*   **Recommended UI:** Command Line Interface (`umst-cli`).
*   **Compute Level:** Nano-Compute.
