#RUN rustup update \
#    && cargo update \
#    && cargo build --release \
#    && useradd -ms /bin/bash PodRacer \
#    && mkdir /opt/lib \
#    && chown -R PodRacer /opt/ \
#    && chmod -R +wx /opt/
#
#USER PodRacer

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
# Building
RUN cargo build --release
# Setup templates dir, output static dir, copy out podracer and podarch.
RUN mkdir -p /app/server && \
    cp -r server/templates /app/server/templates && \
    cp -r target/release/build/podracer-*/out/web/static /app/server/static && \
    cp target/release/podracer /app/podracer && \
    cp target/release/podarch /app/podarch && \
    mkdir /app/podcasts

FROM debian:stable-slim
# Set ROCKET profile and CONFIG path
ENV ROCKET_PROFILE=docker \
    ROCKET_CONFIG=/app/Rocket.toml
# Again the SSL deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl1.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
# Copy from builder the /app, $CONTAINERUSER is a custom user.
#COPY --from=builder --chown=$CONTAINERUSER:$CONTAINERUSER /app /app
#COPY --chown=$CONTAINERUSER:$CONTAINERUSER Rocket.toml /app/Rocket.toml
COPY --from=builder /app /app
COPY Rocket.toml /app/Rocket.toml
WORKDIR /app
CMD ["/app/podracer"]
