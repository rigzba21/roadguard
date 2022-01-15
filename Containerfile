FROM docker.io/library/rust:1.57.0 as build-env
WORKDIR /app
COPY . /app
RUN rustup update && \
    cargo build --release

FROM docker.io/library/ubuntu:latest
RUN apt-get update -y &&  DEBIAN_FRONTEND=noninteractive apt-get install -y \
    sudo wireguard iproute2
COPY --from=build-env /app/target/release/roadguard /


