FROM rust:slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN rustup override set nightly \
    && rustup update \
    && cargo update \
    && cargo build --release

ENTRYPOINT ["cargo"]
CMD ["run","--release","--bin","podracer"]
