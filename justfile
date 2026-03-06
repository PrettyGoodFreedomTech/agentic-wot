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
