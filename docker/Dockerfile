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
    neovim \
    libc6-dev-i386

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

RUN curl -LO https://software-dl.ti.com/codegen/esd/cgt_public_sw/PRU/2.3.3/ti_cgt_pru_2.3.3_linux_installer_x86.bin && \
    chmod +x ti_cgt_pru_2.3.3_linux_installer_x86.bin && \
    /home/worker/ti_cgt_pru_2.3.3_linux_installer_x86.bin --mode unattended --prefix /usr/share && \
    cd /usr/share/ti-cgt-pru_2.3.3/bin && \
    for i in *; do ln -s $PWD/$i /usr/local/bin/; done && \
    cd - && \
    rm /home/worker/ti_cgt_pru_2.3.3_linux_installer_x86.bin

RUN git clone git://git.ti.com/pru-software-support-package/pru-software-support-package.git --depth=1 --branch=v5.8.0 /usr/share/pru-software-support-package

COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
CMD ["/bin/bash"]
