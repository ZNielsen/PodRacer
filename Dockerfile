FROM rust:slim

RUN apt-get install -y --no-install-recommends pkg-config libssl-dev

COPY . .

RUN rustup override set nightly \
    && rustup update \
    && cargo update \
    && cargo build --release

ENTRYPOINT ["cargo"]
CMD ["run","--release","--bin","podracer"]