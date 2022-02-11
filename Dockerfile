FROM rust:slim

ENV STORAGE_DIR="/var/lib/bernie"
ENV RUST_LOG="error,bernie=info"

WORKDIR /usr/src/bernie

COPY . .

RUN apt-get update && apt-get install -y \
  libopus-dev \
  ffmpeg \
  libtool \
  python3 \
  python3-pip \
  && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir yt-dlp

RUN cargo install --path .

VOLUME [ ${STORAGE_DIR} ]

CMD ["bernie"]
