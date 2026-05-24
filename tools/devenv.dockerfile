FROM rust:1.88-bullseye

RUN rustup target add armv7-unknown-linux-musleabihf
