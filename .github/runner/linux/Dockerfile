# GitHub Actions -runner minipc:lle (x86_64), label minipc-linux. Rakentaa Linux x86_64 -paketin
# (release-linux-x86_64.yml, .github/scripts/paketoi-unix.sh) natiivikäännöksenä.
# Ubuntu 22.04: paketti toimii yhtä vanhoissa jakeluissa kuin Linux ARM64 -paketti
# (ubuntu-22.04-arm). Uudempi pohja nostaisi pelaajalta vaadittavaa glibc-versiota.
# Käyttöönotto ja päivitys: ks. ../compose.yaml.
FROM ubuntu:22.04

ARG RUNNER_VERSION=2.338.2
ARG RUNNER_SHA256=baebf204307bac6193480852732f0aba2ae97b6c5597c244f4d5558390147c40
# Sama kuin työnkulkujen RUSTUP_TOOLCHAIN; työ asentaa muun version tarvittaessa itse.
ARG RUST_VERSION=1.95.0
# Sama kuin minipc:n käyttäjän, jotta vain luettavaksi liitetty tulosavain aukeaa kontissa.
ARG UID=1000

ENV DEBIAN_FRONTEND=noninteractive

# Runnerin omat kirjastot (.NET: ICU, OpenSSL, Kerberos, LTTng), pelin Linux-käännös (ALSA;
# vendored OpenSSL tarvitsee perlin ja makein) ja lisenssilistan python.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl git jq \
        libicu70 libssl3 libkrb5-3 zlib1g liblttng-ust1 \
        build-essential pkg-config perl make libasound2-dev \
        python3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u "${UID}" runner
USER runner
WORKDIR /home/runner

RUN curl -fsSL -o /tmp/runner.tgz \
        "https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" \
    && echo "${RUNNER_SHA256}  /tmp/runner.tgz" | sha256sum -c - \
    && mkdir actions-runner && tar -xzf /tmp/runner.tgz -C actions-runner \
    && rm /tmp/runner.tgz

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal \
        --default-toolchain "${RUST_VERSION}"

# Liitoskohdat luodaan valmiiksi, jotta nimetyt levyt saavat runner-käyttäjän omistajuuden.
RUN mkdir -p asetukset valimuisti .cargo/registry

ENV PATH=/home/runner/.cargo/bin:$PATH \
    CARGO_TARGET_DIR=/home/runner/valimuisti/target

COPY --chown=runner:runner --chmod=755 kaynnista.sh /home/runner/kaynnista.sh
ENTRYPOINT ["/home/runner/kaynnista.sh"]
