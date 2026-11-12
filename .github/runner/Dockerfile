# GitHub Actions -runner minipc:lle (Debian, x86_64). Kääntää ja testaa pelin Linuxilla ja
# Windows-exen cargo-xwinillä samaan MSVC-kohteeseen kuin julkaisijan kone.
# Käyttöönotto ja päivitys: ks. compose.yaml.
FROM debian:trixie-slim

ARG RUNNER_VERSION=2.338.2
ARG RUNNER_SHA256=baebf204307bac6193480852732f0aba2ae97b6c5597c244f4d5558390147c40
ARG XWIN_VERSION=0.23.1
ARG XWIN_SHA256=c492c6dfb7e5ac0eee586b796e0fc950ba13077e7f0bbb046f445d71790d5360
# Palvelimen testit (palvelin/test) tarvitsevat Noden oman SQLiten, jota Debianin paketissa ei ole.
ARG NODE_VERSION=24.15.0
ARG NODE_SHA256=44836872d9aec49f1e6b52a9a922872db9a2b02d235a616a5681b6a85fec8d89
# Sama kuin minipc:n käyttäjän, jotta vain luettavaksi liitetty tulosavain aukeaa kontissa.
ARG UID=1000

# Runnerin omat kirjastot (.NET: ICU, OpenSSL, Kerberos), Linux-käännöksen kirjastot (X11,
# ALSA, vendored OpenSSL tarvitsee perlin ja makein), cargo-xwinin clang ja lld, lisenssilistan
# python, paketin 7-Zip ja julkaisun gh.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl git jq \
        libicu76 libssl3t64 libkrb5-3 zlib1g \
        build-essential pkg-config perl make \
        libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libasound2-dev \
        clang lld llvm \
        python3 7zip gh \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/xwin.tgz \
        "https://github.com/rust-cross/cargo-xwin/releases/download/v${XWIN_VERSION}/cargo-xwin-v${XWIN_VERSION}.x86_64-unknown-linux-musl.tar.gz" \
    && echo "${XWIN_SHA256}  /tmp/xwin.tgz" | sha256sum -c - \
    && tar -xzf /tmp/xwin.tgz -C /usr/local/bin cargo-xwin \
    && rm /tmp/xwin.tgz

RUN curl -fsSL -o /tmp/node.tgz \
        "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.gz" \
    && echo "${NODE_SHA256}  /tmp/node.tgz" | sha256sum -c - \
    && tar -xzf /tmp/node.tgz -C /usr/local --strip-components=1 \
        "node-v${NODE_VERSION}-linux-x64/bin/node" \
    && rm /tmp/node.tgz

RUN useradd -m -u "${UID}" runner
USER runner
WORKDIR /home/runner

RUN curl -fsSL -o /tmp/runner.tgz \
        "https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" \
    && echo "${RUNNER_SHA256}  /tmp/runner.tgz" | sha256sum -c - \
    && mkdir actions-runner && tar -xzf /tmp/runner.tgz -C actions-runner \
    && rm /tmp/runner.tgz

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal \
        --default-toolchain stable -c clippy,rustfmt -t x86_64-pc-windows-msvc

# Liitoskohdat luodaan valmiiksi, jotta nimetyt levyt saavat runner-käyttäjän omistajuuden.
RUN mkdir -p asetukset valimuisti .cargo/registry

ENV PATH=/home/runner/.cargo/bin:$PATH \
    CARGO_TARGET_DIR=/home/runner/valimuisti/target \
    XWIN_CACHE_DIR=/home/runner/valimuisti/xwin \
    XWIN_ACCEPT_LICENSE=1

COPY --chown=runner:runner --chmod=755 kaynnista.sh /home/runner/kaynnista.sh
ENTRYPOINT ["/home/runner/kaynnista.sh"]
