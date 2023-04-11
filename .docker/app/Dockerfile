FROM rust:1-bookworm AS builder
RUN apt-get update && apt-get install -y musl musl-dev musl-tools
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/app
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.* ./
RUN cargo fetch --locked
COPY . .
ARG features=bots
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo install --path . --target=x86_64-unknown-linux-musl --features "$features"

FROM alpine as runner
WORKDIR /usr/src/app
COPY --from=builder /usr/local/cargo/bin/meme-download /usr/local/bin/
ENTRYPOINT ["meme-download"]
