FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:latest

RUN echo "hello this is the build!"

ARG CROSS_DEB_ARCH=arm64

RUN dpkg --add-architecture $CROSS_DEB_ARCH
RUN apt-get update
RUN apt-get install --assume-yes libudev-dev:$CROSS_DEB_ARCH