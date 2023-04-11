FROM rust:1.68-alpine3.17 AS builder
RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/app
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.* ./
RUN cargo fetch --locked
COPY src/ ./src/
ARG features=bots
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN cargo install --path . --target=x86_64-unknown-linux-musl --features "$features"

FROM alpine as runner
RUN apk -U add \
  ffmpeg \
  yt-dlp \
  && echo "Done installing packages"
COPY --from=builder /usr/local/cargo/bin/meme-download /usr/local/bin/
ARG username=app
ARG user_id=1001
ARG group_id=1002
RUN addgroup -g ${group_id} ${username}
RUN adduser -D -u ${user_id} -G ${username} ${username}
USER ${username}
ENTRYPOINT ["meme-download"]
CMD [ "--help" ]
