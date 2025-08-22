FROM hub.aiursoft.cn/rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo 'fn main() {}' > src/main.rs
RUN cargo fetch

COPY . .

RUN cargo build --release --locked

FROM hub.aiursoft.cn/debian:stable-slim

WORKDIR /app

RUN apt-get update && apt-get install ca-certificates ffmpeg -y

RUN mkdir -p /usr/local/share/fonts/custom
COPY --from=builder /app/assets/fonts/SmileySans-Oblique.ttf /usr/local/share/fonts/custom/SmileySans-Oblique.ttf

RUN fc-cache -fv

COPY --from=builder /app/target/release/reporter-cli /app/reporter-cli
COPY --from=builder /app/assets /app/assets

ENTRYPOINT ["/app/reporter-cli"]