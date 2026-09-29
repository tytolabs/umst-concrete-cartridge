# SPDX-License-Identifier: MIT
# UMST MCP server image — multi-stage; binary only in runtime layer.
# CI stages private siblings into the repo root before build (see docker.yml).

FROM rust:bookworm AS build

ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY schema ./schema
COPY schemas ./schemas
COPY calibration ./calibration
COPY datasets ./datasets
COPY governance ./governance
COPY umst-foundations ./umst-foundations
COPY umst-cartridge-api ./umst-cartridge-api
COPY umst-cartridges ./umst-cartridges
COPY umst-chem ./umst-chem
COPY umst-cartridge-registry ./umst-cartridge-registry
COPY umst-manifold ./umst-manifold
COPY umst-semantics ./umst-semantics
COPY umst-ucrs ./umst-ucrs

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
COPY --from=build /build/target/release/umst-mcp /usr/local/bin/umst-mcp
USER nonroot
ENTRYPOINT ["/usr/local/bin/umst-mcp"]
