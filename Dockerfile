# SPDX-License-Identifier: MIT
# UMST MCP server image — multi-stage; binary only in runtime layer.
# Build context: parent of this repo (CI checks out private siblings alongside).

FROM rust:bookworm AS build

ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /build-parent
COPY umst-concrete-cartridge/Cargo.toml umst-concrete-cartridge/Cargo.lock ./umst-concrete-cartridge/
COPY umst-concrete-cartridge/crates ./umst-concrete-cartridge/crates
COPY umst-concrete-cartridge/schema ./umst-concrete-cartridge/schema
COPY umst-concrete-cartridge/schemas ./umst-concrete-cartridge/schemas
COPY umst-concrete-cartridge/calibration ./umst-concrete-cartridge/calibration
COPY umst-concrete-cartridge/datasets ./umst-concrete-cartridge/datasets
COPY umst-concrete-cartridge/governance ./umst-concrete-cartridge/governance
COPY umst-foundations ./umst-foundations
COPY umst-cartridge-api ./umst-cartridge-api
COPY umst-cartridges ./umst-cartridges
COPY umst-chem ./umst-chem
COPY umst-cartridge-registry ./umst-cartridge-registry
COPY umst-manifold ./umst-manifold
COPY umst-semantics ./umst-semantics
COPY umst-ucrs ./umst-ucrs

WORKDIR /build-parent/umst-concrete-cartridge
RUN for attempt in 1 2 3; do \
      cargo fetch && break; \
      echo "cargo fetch attempt ${attempt} failed; retrying in 20s..."; \
      sleep 20; \
    done
RUN for attempt in 1 2 3; do \
      cargo build -p umst-mcp --release && exit 0; \
      echo "cargo build attempt ${attempt} failed; retrying in 20s..."; \
      sleep 20; \
    done; \
    exit 1

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /srv
COPY --from=build /build-parent/umst-concrete-cartridge/target/release/umst-mcp /usr/local/bin/umst-mcp
USER nonroot
ENTRYPOINT ["/usr/local/bin/umst-mcp"]
