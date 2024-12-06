FROM ubuntu:22.04

RUN apt-get update && apt-get -y install \
    tcpdump iproute2 net-tools bridge-utils ipmitool \
    build-essential protobuf-compiler libudev-dev pkg-config libclang-dev libibverbs-dev libpci-dev \
    libcairo2-dev libgirepository1.0-dev python3 python3-pip python3-gi network-manager-dev libibumad-dev libibmad-dev\
    git vim curl pciutils apt-transport-https ca-certificates jq

# Install keyring of k8s
RUN curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.29/deb/Release.key | \
    gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
RUN echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.29/deb/ /' | \
    tee /etc/apt/sources.list.d/kubernetes.list

RUN apt-get update && apt-get install -y kubectl

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -q

# Install tools
RUN $HOME/.cargo/bin/cargo install --git https://github.com/openbce/onm smctl
RUN $HOME/.cargo/bin/cargo install --git https://github.com/openbce/onm hcactl
RUN $HOME/.cargo/bin/cargo install --git https://github.com/openbce/onm xpuctl

ENTRYPOINT ["sh", "-c", "exec tail -f /dev/null"]