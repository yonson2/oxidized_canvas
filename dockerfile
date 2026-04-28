FROM rust:1.90-slim-bookworm as builder

WORKDIR /usr/src/

COPY . .

RUN apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app

COPY --from=builder /usr/src/assets/ /usr/app/assets/
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/oxidized_canvas-cli /usr/app/oxidized_canvas-cli

ENTRYPOINT ["/usr/app/oxidized_canvas-cli"]
CMD ["start"]
