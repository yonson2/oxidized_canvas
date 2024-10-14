FROM rust:1.81-slim as builder

WORKDIR /usr/src/

COPY . .

RUN apt update -y && apt install nodejs npm -y
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt update -y && apt install curl wget -y

WORKDIR /usr/app

COPY --from=builder /usr/src/assets/ /usr/app/assets/
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/oxidized_canvas-cli /usr/app/oxidized_canvas-cli

ENTRYPOINT ["/usr/app/oxidized_canvas-cli"]
CMD ["start"]
