FROM rust:1.66.0-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/PodRacer

COPY . .

RUN rustup override set nightly \
    && rustup update \
    && cargo update \
    && cargo build --release \
    && useradd -ms /bin/bash PodRacer \
    && mkdir /opt/lib \
    && chown -R PodRacer /opt/ \
    && chmod -R +wx /opt/

USER PodRacer

ENTRYPOINT ["cargo"]
CMD ["run","--release","--bin","podracer"]
