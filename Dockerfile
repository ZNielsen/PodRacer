FROM rust:1.66.0-slim as builder
# SSL deps
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
# Move to dir
WORKDIR /podracer
# PodRacer source
COPY . .
# Building, Setup templates dir, output static dir, copy out podracer and podarch.
RUN cargo build --release && \
    mkdir -p /app/server && \
    cp -r server/templates /app/server/templates && \
    cp -r target/release/build/podracer-*/out/web/static /app/server/static && \
    cp target/release/podracer /app/podracer && \
    cp target/release/podarch /app/podarch && \
    mkdir /app/podcasts

FROM debian:stable-slim
# Set ROCKET profile and CONFIG path
ENV ROCKET_PROFILE=docker \
    ROCKET_CONFIG=/app/Rocket.toml \
    CONTAINERUSER=PodRacer
# Again the SSL deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl1.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -ms /bin/bash PodRacer
# Copy from builder the /app, $CONTAINERUSER is a custom user.
COPY --from=builder --chown=$CONTAINERUSER:$CONTAINERUSER /app /app
COPY --chown=$CONTAINERUSER:$CONTAINERUSER Rocket.toml /app/Rocket.toml
USER $CONTAINERUSER
WORKDIR /app
ENTRYPOINT ["/app/podracer"]
