# Magic Carpet

Decentralized list marketplace built on Nostr, with NIP-57 zap payments.

## Quick Start

```sh
# Install dependencies
npm install
cargo install dioxus-cli

# Build Tailwind CSS and serve the website
just dx-tw-build
just dx-web
```

The site runs at `http://localhost:8080`.

## Prerequisites

- Rust (stable, edition 2024)
- Node.js (for Tailwind CSS)
- [just](https://github.com/casey/just) (command runner)
- [dioxus-cli](https://github.com/DioxusLabs/dioxus) (`cargo install dioxus-cli`)

## Project Structure

```
crates/
  carpet-ui/     # Dioxus web UI (Tailwind CSS)
  nostr-lib/     # Nostr client — relay mgmt, DCoSL lists, NIP-57 zaps
  dcosl-core/    # DCoSL protocol types and d-tag generation
  phoenixd-lib/  # PhoenixD Lightning HTTP client (used by nostr-lib for zaps)
```

## Commands

| Command | Description |
|---------|-------------|
| `just dx-web` | Serve website (dev mode, hot reload) |
| `just dx-desktop` | Serve as desktop app |
| `just dx-tw` | Watch & rebuild Tailwind CSS |
| `just dx-tw-build` | Build Tailwind CSS once |
| `just check` | `cargo check` workspace |
| `just test` | Run all tests |
| `just clippy` | Lint workspace |

## Tech Stack

- [Dioxus](https://dioxuslabs.com/) 0.7 — full-stack Rust web framework
- [Tailwind CSS](https://tailwindcss.com/) 4 — styling
- [nostr-sdk](https://github.com/rust-nostr/nostr) 0.44 — Nostr protocol
- [DCoSL](https://github.com/matthiasdebernardini/dcosl-core) — Decentralized Curated Sorted Lists protocol
