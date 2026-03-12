infra-up:
    docker compose up -d

infra-down:
    docker compose down

infra-logs:
    docker compose logs -f

mine n="1":
    docker compose exec bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password -generate {{n}}

fund-bdk address:
    docker compose exec bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password sendtoaddress {{address}} 1
    just mine 1

check:
    cargo check --workspace

test:
    cargo test --workspace

build:
    cargo build --workspace

clippy:
    cargo clippy --workspace

# E2E: full lifecycle with guaranteed teardown
e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'just e2e-down' EXIT
    just e2e-up
    just e2e-run

e2e-up:
    #!/usr/bin/env bash
    set -euo pipefail
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ] || [ "$ARCH" = "amd64" ]; then
        docker compose -f docker-compose.yml -f docker-compose.ci.yml up -d
    else
        docker compose up -d
    fi
    # Wait for bitcoind RPC
    echo "Waiting for bitcoind..."
    READY=0
    for i in $(seq 1 30); do
        if docker compose exec -T bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password getblockchaininfo > /dev/null 2>&1; then
            READY=1; break
        fi
        sleep 1
    done
    if [ "$READY" -ne 1 ]; then echo "ERROR: bitcoind not ready after 30s"; docker compose logs bitcoind; exit 1; fi
    # Bootstrap regtest wallet + mine 101 blocks for mature coinbase
    # Must happen before waiting for esplora: electrs waits for IBD to finish,
    # and regtest with 0 blocks reports initialblockdownload=true.
    docker compose exec -T bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password loadwallet "default" 2>/dev/null \
        || docker compose exec -T bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password createwallet "default"
    docker compose exec -T bitcoind bitcoin-cli -regtest -rpcuser=user -rpcpassword=password -rpcwallet=default -generate 101 > /dev/null
    # Wait for esplora HTTP
    echo "Waiting for esplora..."
    READY=0
    for i in $(seq 1 60); do
        if curl -sf http://localhost:3002/blocks/tip/height > /dev/null 2>&1; then
            READY=1; break
        fi
        sleep 2
    done
    if [ "$READY" -ne 1 ]; then echo "ERROR: esplora not ready after 120s"; docker compose logs esplora; exit 1; fi
    echo "Infrastructure ready"

e2e-run:
    cargo test -p bdk-lib --test e2e -- --ignored --test-threads=1

e2e-down:
    docker compose down -v

# Build Tailwind CSS for carpet-ui (watch mode)
dx-tw:
    npx @tailwindcss/cli -i crates/carpet-ui/input.css -o crates/carpet-ui/assets/tailwind.css --watch

# Build Tailwind CSS once (for CI / before build)
dx-tw-build:
    npx @tailwindcss/cli -i crates/carpet-ui/input.css -o crates/carpet-ui/assets/tailwind.css

# Serve carpet-ui for web development
dx-web:
    dx serve -p carpet-ui --platform web

# Serve carpet-ui as a desktop app
dx-desktop:
    dx serve -p carpet-ui --platform desktop
