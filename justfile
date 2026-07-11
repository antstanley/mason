# mason — one wall, every brick

# run mortar + the web app together
dev:
    #!/usr/bin/env bash
    trap 'kill 0' EXIT
    (cd server && cargo run) &
    (cd web && pnpm dev) &
    wait

# full test + check suite
test:
    cd server && cargo nextest run
    cd web && pnpm check:ci

lint:
    cd web && pnpm oxlint src
    cd web && pnpm knip
    cd server && cargo clippy -- -D warnings

fmt:
    cd web && pnpm oxfmt src
    cd server && cargo fmt

fmt-check:
    cd web && pnpm oxfmt --check src
    cd server && cargo fmt --check

# the video rule: autoplay must never appear in web source
guard-autoplay:
    ! git grep -n 'autoplay' web/src
