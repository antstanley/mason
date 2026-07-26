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
    cd web && pnpm test

# service-worker smoke: the static build driven end to end in chromium
test-e2e: build
    cd web && pnpm test:e2e

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
    # A DENYLIST, not an allowlist. This recipe used to name the paths it
    # scanned, which meant every new tracked directory was silently unguarded
    # until somebody remembered to add it: .specs/ went a whole spec set
    # unchecked that way, and the gate's exit 0 was false assurance for it.
    # Scanning everything and excluding the generated trees cannot fail that way.
    # -I skips binaries, so a png or a wasm blob holding the byte sequence by
    # chance is not a finding. Filesystem grep, not jj: a new, unsnapshotted
    # file must be caught too.
    # build the pattern from bytes so this recipe holds no literal em dash
    dash=$(printf '\xe2\x80\x94')
    if grep -rlI "$dash" . \
        --exclude-dir=.git --exclude-dir=.jj --exclude-dir=node_modules \
        --exclude-dir=target --exclude-dir=build --exclude-dir=dist \
        --exclude-dir=.svelte-kit --exclude-dir=pkg --exclude-dir=.impeccable; then
        echo "guard-dashes: found a U+2014 em dash in tracked source" >&2
        exit 1
    fi

# rust-toolchain.toml is the one place the channel is pinned, and CI parses it
# rather than repeating it. What a parse cannot check is the OTHER version:
# [workspace.package] rust-version declares the MSRV at minor granularity, and
# nothing stops it drifting a minor behind the channel and quietly promising
# support for a compiler nobody builds with. This asserts the channel satisfies
# the MSRV it claims.
guard-toolchain:
    #!/usr/bin/env bash
    set -euo pipefail
    channel=$(sed -n 's/^channel[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' rust-toolchain.toml)
    msrv=$(sed -n 's/^rust-version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' server/Cargo.toml)
    if [ -z "$channel" ] || [ -z "$msrv" ]; then
        echo "guard-toolchain: could not read channel ($channel) or rust-version ($msrv)" >&2
        exit 1
    fi
    # compare on major.minor; the channel may carry a patch the MSRV does not
    if [ "${channel%.*}" != "$msrv" ] && [ "$channel" != "$msrv" ]; then
        echo "guard-toolchain: rust-toolchain.toml pins $channel but server/Cargo.toml declares MSRV $msrv" >&2
        exit 1
    fi

# This is the ONE list of gates; the push wrapper below calls it rather than
# repeating it, so there is no second copy to drift out of step.
# test-e2e and test-wasm stay out on purpose: both need a real browser, which
# takes the run past a minute, and a gate that slow is one people learn to skip
# around. CI still runs those two lanes, and CI is the authority.
# Ordered cheapest first, measured rather than guessed: the two greps are ~0.2s
# each, fmt-check ~1s, lint ~2s (clippy, and minutes on a cold target dir), test
# ~3.5s. An em dash used to cost a full clippy pass before anything reported it.
[doc('the local gate: the guards, fmt-check, lint, test. cheapest first')]
check: guard-dashes guard-autoplay guard-toolchain fmt-check lint test

# jj has no hook system, and `jj git push` does not fire the colocated repo's
# .git/hooks/pre-push either (verified: only a raw `git push` triggers it, and
# this repo forbids raw git). So nothing can intercept a push here; this recipe
# IS the gate, by being the shorter and only documented way to push.
[doc('push through the gate: run `just check`, then jj git push')]
push *ARGS: check
    jj git push {{ARGS}}

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
