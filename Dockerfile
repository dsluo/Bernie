FROM rust:slim as builder

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
    libopus-dev \
    pkg-config \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/bernie

COPY . .

RUN cargo build --release --bin bernie

FROM debian:stable-slim as final

ENV STORAGE_DIR="/var/lib/bernie"
ENV RUST_LOG="error,bernie=info"

VOLUME ["${STORAGE_DIR}"]

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    python3 \
    ca-certificates \
    ffmpeg \
  && python3 -c \
    "import urllib.request;\
    urllib.request.urlretrieve(\
      'https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp',\
      '/usr/local/bin/yt-dlp'\
    )" \
  && chmod a+rx /usr/local/bin/yt-dlp \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/bernie/target/release/bernie /usr/local/bin

ENTRYPOINT ["/usr/local/bin/bernie"]
