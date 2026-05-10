<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Dataset provenance

CSV filenames are stable (`dataset_d*.csv`, …) so external links stay valid. Rows were copied verbatim (headers preserved) from `datasets/` into this directory with lowercased filenames. Licence and attribution for each corpus are in **`DATA_LICENSE`**.

## dataset_d1.csv

- **Source path:** `datasets/dataset_D1.csv`
- **Rows (incl. header):** 1031 (1030 mixes)
- **Primary reference:** I-Cheng Yeh (1998), UCI MLR concrete compressive strength dataset
- **Columns:** `cement, slag, fly_ash, water, superplasticizer, coarse_agg, fine_agg, age, strength, source, temperature, humidity` (masses kg/m³, age days, strength MPa unless noted otherwise in UCI docs)
- **Calibration profile:** `uci_d1`
- **SHA-256:** `8a378ad045929d4f398acc8261cef0901cd90c2b799f2f6cbfb582adcf55a6d3`

## dataset_d2.csv

- **Source path:** `datasets/dataset_D2.csv`
- **Rows (incl. header):** 4892 (4891 mixes)
- **Primary corpus:** Eindhoven University of Technology and TNO. *Non-destructive Estimation of Concrete Compressive Strength.* Zenodo record **14921019**, DOI **10.5281/zenodo.14921019**, CC-BY 4.0
- **Subset in this cartridge:** **NDT** — calibration profile **`zenodo_ndt`**
- **SHA-256:** `2eb70e4f347d156054d86992c7425c8289a167cae35058f3e329f92bdec1ce5d`

## dataset_d3.csv

- **Source path:** `datasets/dataset_D3.csv`
- **Rows (incl. header):** 2781 (2780 mixes)
- **Primary corpus:** Zenodo **14921019** (TU/e + TNO), CC-BY 4.0 — same record as `dataset_d2.csv`
- **Subset in this cartridge:** **SonReb** — calibration profile **`zenodo_sonreb`**
- **SHA-256:** `079004d76e9948f04a7d9745d492d5a209097dfff06d99a0b7c2858864d64a6f`

## dataset_d4.csv

- **Source path:** `datasets/dataset_D4.csv`
- **Rows (incl. header):** 7446 (7445 mixes)
- **Primary corpus:** Zenodo **14921019** (TU/e + TNO), CC-BY 4.0 — same record as `dataset_d2.csv`
- **Subset in this cartridge:** **RH** — calibration profile **`zenodo_rh`**
- **SHA-256:** `e0f0bc10fac55cf25144022db2e1e3989cf3dfe107fccb995bbab7380f9a3b20`

## dataset_uhpc.csv

- **Source path:** `datasets/dataset_uhpc.csv`
- **Rows (incl. header):** 501
- **Calibration profile:** `uhpc` (Boundary)
- **SHA-256:** `a3942ba5b6748d4d83786c16d7dced32fc58c660b336c90bc06c055f1c1cca58`

## dataset_highscm.csv

- **Source path:** `datasets/dataset_highscm.csv`
- **Rows (incl. header):** 501
- **Calibration profile:** `highscm`
- **SHA-256:** `7d651fa6d04dcc28e043de389fac71f5158f56b72e0e9e8ba249f20c45d56291`

## dataset_selfheal.csv

- **Source path:** `datasets/dataset_selfheal.csv`
- **Rows (incl. header):** 501
- **Calibration profile:** `selfheal` (Boundary)
- **SHA-256:** `7ad647899db864a1f821edd359bb71c87f7235df79284fb74ba80cd359dd5105`
