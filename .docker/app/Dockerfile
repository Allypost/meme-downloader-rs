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
  busybox-openrc \
  && echo "Done installing packages"
RUN apk update && apk add --no-cache fontconfig curl curl-dev && \
  mkdir -p /usr/share && \
  cd /usr/share \
  && curl -L https://github.com/Overbryd/docker-phantomjs-alpine/releases/download/2.11/phantomjs-alpine-x86_64.tar.bz2 | tar xj \
  && ln -s /usr/share/phantomjs/phantomjs /usr/bin/phantomjs \
  && phantomjs --version
RUN rc-update add crond
COPY ./.docker/app/update-packages.sh /usr/src/update-packages.sh
RUN echo "0 */2 * * * /bin/sh /usr/src/update-packages.sh" >> /var/spool/cron/crontabs/root
COPY --from=builder /usr/local/cargo/bin/meme-download /usr/local/bin/
ARG username=app
ARG user_id=1001
ARG group_id=1002
RUN addgroup -g ${group_id} ${username}
RUN adduser -D -u ${user_id} -G ${username} ${username}
USER ${username}
ENTRYPOINT ["meme-download"]
CMD [ "--help" ]
