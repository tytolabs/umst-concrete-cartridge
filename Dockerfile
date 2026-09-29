# SPDX-License-Identifier: MIT
# UMST MCP server image — multi-stage; binary only in runtime layer.

FROM rust:bookworm AS build

# crates.io HTTP/2 framing flakes in CI Docker builds (burn-core download).
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

WORKDIR /app
# Cargo.lock is not committed (workspace gitignore); resolve deps during image build.
COPY Cargo.toml ./
COPY crates ./crates
COPY schema ./schema
COPY schemas ./schemas
COPY calibration ./calibration
COPY datasets ./datasets
COPY governance ./governance

RUN --mount=type=secret,id=git_token \
    git config --global url."https://x-access-token:$(cat /run/secrets/git_token)@github.com/".insteadOf "https://github.com/" && \
    for attempt in 1 2 3; do \
      cargo fetch && break; \
      echo "cargo fetch attempt ${attempt} failed; retrying in 20s..."; \
      sleep 20; \
    done
RUN --mount=type=secret,id=git_token \
    git config --global url."https://x-access-token:$(cat /run/secrets/git_token)@github.com/".insteadOf "https://github.com/" && \
    for attempt in 1 2 3; do \
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
