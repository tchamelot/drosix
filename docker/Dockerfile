FROM debian:buster-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y \
    --no-install-recommends \
    # ca-certificates \
    # buildroot core
    sed \
    make \
    binutils \
    build-essential \
    gcc \
    g++ \
    bash \
    patch \
    gzip \
    bzip2 \
    perl \
    tar \
    cpio \
    unzip \
    file \
    bc \
    wget \
    # buildroot optional
    python3 \
    libncurses5-dev \
    git \
    mercurial \
    rsync \
    ssh \
    # rust
    curl \
    # misc
    locales \
    whois \
    neovim 

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates

# Sometimes Buildroot need proper locale, e.g. when using a toolchain
# based on glibc.
RUN locale-gen en_US.utf8

ARG USER_ID
ARG GROUP_ID

RUN addgroup --gid $GROUP_ID worker
RUN useradd -u $USER_ID -g $GROUP_ID -ms /bin/bash worker && \
    chown -R worker:worker /home/worker
USER worker
WORKDIR /home/worker
ENV HOME /home/worker

RUN git clone git://git.buildroot.net/buildroot --depth=1 --branch=2020.08 /home/worker/buildroot

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y \
    --default-toolchain nightly\
    --target wasm32-unknown-unknown \
    --target armv7-unknown-linux-gnueabihf \
    --profile minimal \
    -c clippy rustfmt rust-src rls

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN echo '[target.armv7-unknown-linux-gnueabihf]\nlinker = "/home/worker/output/host/bin/arm-drosix-linux-gnueabihf-gcc"' >> $HOME/.cargo/config

ENV BR2_EXTERNAL=/home/worker/drosix/br2-drosix-tree
ENV TARGET_CC=/home/worker/output/host/bin/arm-drosix-linux-gnueabihf-gcc
ENV SYSROOT=/home/worker/output/staging
ENV PKG_CONFIG=/home/worker/output/host/bin/pkg-config
ENV PKG_CONFIG_ALLOW_CROSS=1 
ENV PKG_CONFIG_LIBDIR=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig
ENV PKG_CONFIG_SYSROOT_DIR=$SYSROOT

# Work around to change permission on the mounted volume in /home/worker
USER root
COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
CMD ["/bin/bash"]
