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
RUN apk add \
  ffmpeg \
  python3 \
  && echo "Done installing packages"
RUN \
  wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp \
  && chmod +x /usr/local/bin/yt-dlp \
  && ln -s /usr/local/bin/yt-dlp /usr/local/bin/youtube-dl \
  && echo "yt-dlp version: $(yt-dlp --version)"
COPY --from=builder /usr/local/cargo/bin/meme-download /usr/local/bin/
ARG username=app
ARG user_id=1001
ARG group_id=1002
RUN addgroup -g ${group_id} ${username}
RUN adduser -D -u ${user_id} -G ${username} ${username}
USER ${username}
ENTRYPOINT ["meme-download"]
