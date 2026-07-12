# SPDX-License-Identifier: MIT
# UMST MCP server image — multi-stage; binary only in runtime layer.

FROM rust:bookworm AS build

# crates.io HTTP/2 framing flakes in CI Docker builds (burn-core download).
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false

WORKDIR /app
# Cargo.lock is not committed (workspace gitignore); resolve deps during image build.
COPY Cargo.toml ./
COPY crates ./crates
COPY schema ./schema
COPY calibration ./calibration
COPY datasets ./datasets
COPY governance ./governance

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
COPY --from=build /app/target/release/umst-mcp /usr/local/bin/umst-mcp
USER nonroot
ENTRYPOINT ["/usr/local/bin/umst-mcp"]
