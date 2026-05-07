# UMST Concrete Cartridge

> The First Thermodynamic Science Cartridge for the UMST Framework.

`umst-concrete-cartridge` is the physical constitutive engine that mounts directly onto the **UMST Manifold**. Written entirely in pure, functional `burn` tensor operations, it maps topological scalar features into real-world material behaviors.

## 🔬 Embedded Physics Models

The cartridge solves a highly non-linear, multi-physics interaction graph simultaneously across the Cellular Sheaf.

| Module | Core Physics | Output Tensors |
|--------|--------------|----------------|
| **Colloidal** | DLVO Theory, Zeta Potential | Flocculation Multiplier |
| **Rheology** | Chateau-Ovarlez, YODEL | Yield Stress, Viscosity |
| **Printability** | Roussel Constraints | Buildability, Extrudability |
| **Strength** | Jennings CM-II | Compressive Strength (MPa) |
| **Fracture** | Ulm Micromechanics | Fracture Toughness ($K_{Ic}$) |
| **Durability** | Transport, Freeze-Thaw | Diffusivity, Internal RH |
| **Lifecycle** | Creep, Autogenous Shrinkage | Compliance |

## ⚙️ Functional Topologies

The entire engine implements the `IScienceCartridge` trait. It completely bypasses dense 4D arrays, gathering and scattering heat, stress, and chemical flow strictly across the `$B_1$` edge matrices of the UMST Manifold to guarantee absolute mass and energy conservation.

```mermaid
flowchart LR
    classDef domain fill:#1a1a2e,stroke:#e94560,stroke-width:2px,color:#fff
    classDef prop fill:#16213e,stroke:#0f3460,stroke-width:2px,color:#fff

    S[Scalar Features]:::domain --> Nano[Nano / Colloidal]:::prop
    S --> Chem[Chemo-Water]:::prop
    
    Nano --> Rheo[Rheology]:::prop
    Chem --> Thermo[Thermodynamics]:::prop
    
    Rheo --> Print[Printability Safety]:::domain
    Thermo --> Strength[Jennings CM-II]:::domain
```

## 📜 License

Apache License 2.0. Copyright Studio Tyto.
