# phoenixd Docker management and API commands

compose := "docker compose -f docker-compose.phoenixd.yml"
vault   := "phoenixd"

# Start phoenixd
up:
    {{compose}} up -d
    @echo "phoenixd starting... run 'just -f phoenixd.justfile backup-to-1p' after first start"

# Stop phoenixd
down:
    {{compose}} down

# Stop and destroy volume (CAUTION: deletes seed + channels)
nuke:
    {{compose}} down -v

# Show logs
logs *args:
    {{compose}} logs {{args}}

# Follow logs
follow:
    {{compose}} logs -f

# Create dedicated 1Password vault (run once, idempotent)
setup-1p:
    op vault get {{vault}} >/dev/null 2>&1 || op vault create {{vault}}
    @echo "1Password vault '{{vault}}' ready"

# Backup seed + password to 1Password (idempotent — updates if item exists)
backup-to-1p:
    #!/usr/bin/env bash
    set -euo pipefail
    SEED=$(docker exec phoenixd cat /phoenix/.phoenix/seed.dat)
    PW=$(docker exec phoenixd cat /phoenix/.phoenix/phoenix.conf | grep http-password | cut -d= -f2)
    NODE_ID=$(docker exec phoenixd /phoenix/phoenix-cli getinfo | jq -r .nodeId)
    if op item get "phoenixd mainnet" --vault="{{vault}}" >/dev/null 2>&1; then
      op item edit "phoenixd mainnet" --vault="{{vault}}" \
        "seed=${SEED}" \
        "http-password=${PW}" \
        "node-id=${NODE_ID}"
      echo "Updated existing item in 1Password vault '{{vault}}'"
    else
      op item create \
        --category="Secure Note" \
        --vault="{{vault}}" \
        --title="phoenixd mainnet" \
        "seed=${SEED}" \
        "http-password=${PW}" \
        "node-id=${NODE_ID}"
      echo "Created new item in 1Password vault '{{vault}}'"
    fi

# Show the seed from 1Password
seed:
    @op item get "phoenixd mainnet" --vault={{vault}} --fields seed --reveal

# Verify running node ID matches 1Password backup
verify-node:
    #!/usr/bin/env bash
    set -euo pipefail
    LIVE=$(docker exec phoenixd /phoenix/phoenix-cli getinfo | jq -r .nodeId)
    STORED=$(op item get "phoenixd mainnet" --vault={{vault}} --fields node-id --reveal)
    if [ "$LIVE" = "$STORED" ]; then
      echo "✅ Node ID matches: $LIVE"
    else
      echo "❌ MISMATCH"
      echo "  Running: $LIVE"
      echo "  1Password: $STORED"
      exit 1
    fi

# --- API commands via phoenix-cli (password auto-read from config) ---

# Shorthand for docker exec
cli := "docker exec phoenixd /phoenix/phoenix-cli"

# Get node info
info:
    {{cli}} getinfo

# Get balance
balance:
    {{cli}} getbalance

# List channels
channels:
    {{cli}} listchannels

# Estimate liquidity fees for an amount
estimate-fees amount:
    {{cli}} estimateliquidityfees --amountSat={{amount}}

# Create invoice: just -f phoenixd.justfile invoice 1000 "test payment"
invoice amount desc="magic-carpet":
    {{cli}} createinvoice --amountSat={{amount}} --desc="{{desc}}"

# Create invoice and display as QR code: just -f phoenixd.justfile qr-invoice 3000 "initial funding"
qr-invoice amount desc="magic-carpet":
    #!/usr/bin/env bash
    set -euo pipefail
    BOLT11=$({{cli}} createinvoice --amountSat={{amount}} --desc="{{desc}}" | jq -r .serialized)
    echo "$BOLT11"
    echo "$BOLT11" | qrencode -t ANSIUTF8

# Create a reusable offer (amount optional)
offer desc="magic-carpet" amount="":
    #!/usr/bin/env bash
    ARGS="--desc={{desc}}"
    [ -n "{{amount}}" ] && ARGS="$ARGS --amountSat={{amount}}"
    {{cli}} createoffer $ARGS

# Get default offer (reusable invoice)
get-offer:
    {{cli}} getoffer

# Get Lightning address (requires a channel)
ln-address:
    {{cli}} getlnaddress

# Pay a BOLT11 invoice
pay bolt11:
    {{cli}} payinvoice --invoice={{bolt11}}

# Scan a QR code via webcam and pay the invoice
scan-pay:
    #!/usr/bin/env bash
    set -euo pipefail
    BOLT11=$(python3 scripts/scan_qr.py)
    echo ""
    echo "Invoice: $BOLT11"
    read -p "Pay this invoice? [y/N] " confirm
    [[ "$confirm" =~ ^[Yy]$ ]] || { echo "Cancelled."; exit 0; }
    {{cli}} payinvoice --invoice="$BOLT11"

# Pay a Lightning offer
pay-offer offer amount:
    {{cli}} payoffer --offer={{offer}} --amountSat={{amount}}

# List incoming payments
incoming:
    {{cli}} listincomingpayments

# Get incoming payment by hash
incoming-payment hash:
    {{cli}} getincomingpayment --hash={{hash}}

# List outgoing payments
outgoing:
    {{cli}} listoutgoingpayments

# Get outgoing payment by UUID or hash
outgoing-payment id:
    {{cli}} getoutgoingpayment --uuid={{id}}
