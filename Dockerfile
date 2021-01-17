# Nightly image needed for Rocket v0.4.6
ARG BUILD_IMAGE=ekidd/rust-musl-builder:nightly-2021-01-01
ARG RUNNER_IMAGE=alpine:latest

FROM ${BUILD_IMAGE} AS build

RUN USER=rust:rust cargo init --name spaghetti
ADD --chown=rust:rust Cargo.toml Cargo.lock ./
RUN USER=rust:rust cargo build --release

ADD --chown=rust:rust src ./src
# cargo requires touching file for recompilation
RUN USER=rust:rust touch src/main.rs
RUN USER=rust:rust cargo build --release


FROM ${RUNNER_IMAGE}

#RUN apk --no-cache add ca-certificates
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/server \
    /usr/local/bin
ADD Rocket.toml ./

ENTRYPOINT [ "/usr/local/bin/server" ] 
