FROM rust:1.71 AS builder
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN rustup toolchain install nightly
RUN rustup target add --toolchain nightly x86_64-unknown-linux-musl
WORKDIR /usr/src/app
COPY . .
ARG features=bots
ENV RUSTFLAGS='-C target-feature=+crt-static'
RUN rustup run nightly cargo build \
  -Z unstable-options \
  --release \
  --target=x86_64-unknown-linux-musl \
  --bin 'meme-downloader' \
  --out-dir '.out' \
  --features "$features"

FROM ubuntu:rolling as runner
RUN apt-get update && apt-get install -y \
  ffmpeg \
  sudo \
  curl \
  fontconfig \
  bzip2 \
  python3 \
  && echo "Done installing packages"
ENV OPENSSL_CONF=/dev/null
RUN mkdir -p /usr/share \
  && cd /usr/share \
  && curl -L https://bitbucket.org/ariya/phantomjs/downloads/phantomjs-2.1.1-linux-x86_64.tar.bz2 | tar xj \
  && ln -s /usr/share/phantomjs-2.1.1-linux-x86_64/bin/phantomjs /usr/bin/phantomjs \
  && phantomjs --version
RUN apt-get update && apt-get install -y \
  xattr \
  && echo "Done installing packages"
COPY ./.docker/app/update_yt-dlp.sh /usr/bin/update_yt-dlp.sh
RUN chmod a=rx,u+s /usr/bin/update_yt-dlp.sh
RUN /usr/bin/update_yt-dlp.sh
COPY --from=builder /usr/src/app/.out/meme-downloader /usr/local/bin/
ARG username=app
ARG user_id=1000
ARG group_id=1000
RUN userdel --remove ubuntu; groupdel ubuntu; echo "Deleted default ubuntu user"
RUN groupadd --gid ${group_id} ${username}
RUN useradd --create-home --uid ${user_id} --gid ${group_id} ${username}
RUN echo -e "${username}\tALL= NOPASSWD: /usr/sbin/crond, /usr/bin/update_yt-dlp.sh" > "/etc/sudoers.d/${username}" \
  && chown -c root:root /etc/sudoers \
  && chmod -c 0440 /etc/sudoers
USER ${username}
LABEL org.opencontainers.image.authors="me@allypost.net"
ENTRYPOINT ["meme-downloader"]
CMD [ "--help" ]
