check:
    cargo check --workspace

test:
    cargo test --workspace

build:
    cargo build --workspace

clippy:
    cargo clippy --workspace

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
