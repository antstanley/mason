# mason: one wall, every brick
# default mode is LOCAL: no server, mortar runs as wasm in a service worker

# build the wasm feed engine into the web app
wasm:
    cd server && wasm-pack build crates/mortar-wasm --target web --no-pack \
        --out-dir ../../../web/src/lib/mortar-wasm/pkg

# local mode dev: wasm service worker serves the feed, no Rust server
dev: wasm
    cd web && pnpm dev

# server mode dev: native mortar + the SPA pointed at it over CORS
dev-server:
    #!/usr/bin/env bash
    trap 'kill 0' EXIT
    (cd server && cargo run -p mortar-server) &
    server=$!
    (cd web && PUBLIC_MASON_SERVER_URL=http://localhost:8787 pnpm dev) &
    web=$!
    # exit as soon as the FIRST child dies, then the EXIT trap tears the other
    # down; a bare `wait` would block until both exit and leave a survivor
    # lingering. `wait -n` would be cleaner but needs bash 4+, and macOS ships
    # bash 3.2, so poll the two pids instead.
    while kill -0 "$server" 2>/dev/null && kill -0 "$web" 2>/dev/null; do
        sleep 1
    done

# fully static production build (local mode) → web/build/
build: wasm
    cd web && pnpm build

# full test + check suite
test:
    cd server && cargo nextest run
    cd web && pnpm check:ci

# run the wasm-only Rust paths (transport, timers, throttle) for real in a
# headless browser; wasm-pack fetches a matching chromedriver if none is found
test-wasm:
    cd server && wasm-pack test --headless --chrome crates/mortar-core

lint:
    cd web && pnpm oxlint src
    cd web && pnpm knip
    cd server && cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cd web && pnpm oxfmt src
    cd server && cargo fmt --all

fmt-check:
    cd web && pnpm oxfmt --check src
    cd server && cargo fmt --all --check

# the video rule: no autoplay attributes and no programmatic play() outside the
# one sanctioned, click-gated player. filesystem grep (not git grep) so new,
# unsnapshotted files in this jj repo are seen too.
guard-autoplay:
    #!/usr/bin/env bash
    set -euo pipefail
    # no autoplay attribute or autostart flag anywhere in web source
    if grep -rniE 'autoplay|autostartload' web/src; then
        echo "guard-autoplay: found an autoplay reference in web/src" >&2
        exit 1
    fi
    # the only sanctioned .play() is VideoPlayer.svelte, gated behind a click
    if grep -rnF '.play(' web/src --exclude=VideoPlayer.svelte; then
        echo "guard-autoplay: found a programmatic .play( outside VideoPlayer.svelte" >&2
        exit 1
    fi

# no em dashes anywhere in tracked source, docs, or config (U+2014)
guard-dashes:
    #!/usr/bin/env bash
    set -euo pipefail
    # build the pattern from bytes so this recipe holds no literal em dash
    dash=$(printf '\xe2\x80\x94')
    if grep -rl "$dash" \
        --exclude-dir=.git --exclude-dir=.jj --exclude-dir=node_modules \
        --exclude-dir=target --exclude-dir=build \
        web/src web/vite.config.ts web/knip.json web/package.json web/tsconfig.json \
        server README.md AGENTS.md PRODUCT.md CHANGELOG.md .changeset justfile; then
        echo "guard-dashes: found a U+2014 em dash in tracked source" >&2
        exit 1
    fi

# deploy to AWS via blogwright (S3 + CloudFront, MicroVM build)
deploy env='production': wasm
    cd web && pnpm exec blogwright deploy {{env}}

# one-time infra creation (needs AWS credentials)
bootstrap env='production':
    cd web && pnpm exec blogwright bootstrap {{env}}

# one-time PR-preview stack creation (domain = Route53 hosted zone, not committed)
bootstrap-preview domain:
    cd web && pnpm exec blogwright preview bootstrap --domain {{domain}}

# reclaim disk (cargo target grows to ~3GB)
clean:
    cd server && cargo clean
