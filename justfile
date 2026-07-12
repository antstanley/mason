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
    (cd web && PUBLIC_MASON_SERVER_URL=http://localhost:8787 pnpm dev) &
    wait

# fully static production build (local mode) → web/build/
build: wasm
    cd web && pnpm build

# full test + check suite
test:
    cd server && cargo nextest run
    cd web && pnpm check:ci

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

# the video rule: autoplay must never appear in web source
guard-autoplay:
    ! git grep -n 'autoplay' web/src

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
