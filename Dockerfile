# SPDX-License-Identifier: MIT
# UMST MCP server image — multi-stage; binary only in runtime layer.

FROM rust:bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY schema ./schema
COPY calibration ./calibration
COPY datasets ./datasets

RUN cargo build -p umst-mcp --release

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /srv
COPY --from=build /app/target/release/umst-mcp /usr/local/bin/umst-mcp
USER nonroot
ENTRYPOINT ["/usr/local/bin/umst-mcp"]
