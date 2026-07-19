//! The Rust half of the wire-contract drift guard (issue #33).
//!
//! `web/src/lib/types.ts` hand-mirrors mortar's serde output, and nothing used
//! to catch a rename on either side: tsc cannot see Rust and nextest cannot see
//! TS. This test pins the contract in one committed fixture,
//! `tests/fixtures/contract.json`: a canonical instance of every `Brick` kind,
//! the `FeedResponse` shapes, every `AppError` envelope in both build modes,
//! and the query vocabulary (mode / intent). The web side re-checks the same
//! file at type level in `web/src/lib/contract-check.ts`, so:
//!
//! - a Rust-side rename changes the serialization, and this test fails until
//!   the fixture is regenerated (which then fails tsc until types.ts follows);
//! - a TS-side rename fails tsc against the committed fixture directly.
//!
//! Regenerate after an intentional wire change with:
//!
//! ```sh
//! UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract
//! ```
//!
//! Vocabulary (brick kinds, error codes, video sources, query tokens) rides in
//! the fixture as OBJECT KEYS, not string values: tsc widens JSON string
//! values to `string`, but object keys stay literal, so `keyof` on the web
//! side sees the exact tokens.

use std::path::Path;

use mortar_core::error::AppError;
use mortar_core::feed::FeedIntent;
use mortar_core::mode::Mode;
use mortar_core::model::{
    AspectRatio, Author, BlogBrick, Blur, Brick, CaptionTrack, ExternalEmbed, FeedResponse,
    ImageEmbed, PostBrick, Publication, VideoBrick, VideoSource,
};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

/// Every Brick kind the fixture must cover, in one place. `kind_key` indexes
/// into this array, and `contract()` asserts the fixture's brick map carries
/// exactly these keys, so the array's length is the fixture's coverage.
const ALL_KINDS: [&str; 3] = ["post", "blog", "video"];

/// The one key per Brick kind. The forcing chain for a new variant: the match
/// must gain an arm (exhaustiveness), the arm must index a NEW slot of
/// `ALL_KINDS` (an out-of-bounds constant index fails the build, via
/// deny-by-default `unconditional_panic` when the test compiles and clippy's
/// `out_of_bounds_indexing` in `just lint`, so the array must grow), and the
/// key-set assert in `contract()` then fails until `bricks()` actually
/// contributes canonical instances under the new key. The key must also equal
/// the serialized `kind` tag, which `bricks()` asserts.
fn kind_key(brick: &Brick) -> &'static str {
    match brick {
        Brick::Post(_) => ALL_KINDS[0],
        Brick::Blog(_) => ALL_KINDS[1],
        Brick::Video(_) => ALL_KINDS[2],
    }
}

/// An author with every optional field present.
fn author_full() -> Author {
    Author {
        did: "did:plc:mason".into(),
        handle: "mason.example.com".into(),
        display_name: Some("mason".into()),
        avatar: Some("https://cdn.example.com/avatar.jpg".into()),
    }
}

/// An author with every optional field absent, to pin the null side.
fn author_bare() -> Author {
    Author {
        did: "did:plc:bare".into(),
        handle: "bare.example.com".into(),
        display_name: None,
        avatar: None,
    }
}

/// Per kind, a `full` instance (every optional field present, so the web side
/// can compare its exact key set against the TS interface) and a `bare` one
/// (every optional field absent or empty, pinning null-vs-absent modeling).
/// Deliberately maximal, not realistic: a streamplace brick never carries a
/// blur in practice, but the contract needs every field on the wire once.
fn bricks() -> Vec<(Brick, &'static str, Value)> {
    let post_full = Brick::Post(PostBrick {
        id: "at://did:plc:mason/app.bsky.feed.post/full".into(),
        url: "https://bsky.app/profile/mason.example.com/post/full".into(),
        author: author_full(),
        text: "one wall, every brick".into(),
        created_at: "2026-01-02T03:04:05.000Z".into(),
        like_count: 12,
        repost_count: 3,
        images: vec![ImageEmbed {
            src: "https://cdn.example.com/brick.jpg".into(),
            alt: "a brick".into(),
            aspect_ratio: Some(AspectRatio {
                width: 4,
                height: 3,
            }),
        }],
        external: Some(ExternalEmbed {
            uri: "https://example.com/article".into(),
            title: "an article".into(),
            description: "worth a read".into(),
            thumb: Some("https://cdn.example.com/thumb.jpg".into()),
        }),
        blur: Some(Blur {
            label: "!warn".into(),
        }),
    });
    let post_bare = Brick::Post(PostBrick {
        id: "at://did:plc:bare/app.bsky.feed.post/bare".into(),
        url: "https://bsky.app/profile/bare.example.com/post/bare".into(),
        author: author_bare(),
        text: "just words".into(),
        created_at: "2026-01-02T03:04:05.000Z".into(),
        like_count: 0,
        repost_count: 0,
        images: vec![],
        external: None,
        blur: None,
    });
    let blog_full = Brick::Blog(BlogBrick {
        id: "at://did:plc:mason/site.standard.document/full".into(),
        url: "https://blog.example.com/one-wall".into(),
        author: author_full(),
        title: "one wall".into(),
        description: Some("every brick".into()),
        cover_image: Some("https://cdn.example.com/cover.jpg".into()),
        publication: Publication {
            name: "the hod".into(),
            url: "https://blog.example.com".into(),
            icon: Some("https://blog.example.com/icon.png".into()),
        },
        tags: vec!["masonry".into(), "atproto".into()],
        published_at: "2026-01-02T03:04:05.000Z".into(),
    });
    let blog_bare = Brick::Blog(BlogBrick {
        id: "at://did:plc:bare/site.standard.document/bare".into(),
        url: "https://blog.example.com/bare".into(),
        author: author_bare(),
        title: "bare".into(),
        description: None,
        cover_image: None,
        publication: Publication {
            name: "the hod".into(),
            url: "https://blog.example.com".into(),
            icon: None,
        },
        tags: vec![],
        published_at: "2026-01-02T03:04:05.000Z".into(),
    });
    let video_full = Brick::Video(VideoBrick {
        id: "at://did:plc:mason/place.stream.livestream/full".into(),
        url: "https://stream.place/mason.example.com".into(),
        author: author_full(),
        title: "laying bricks live".into(),
        poster: Some("https://cdn.example.com/poster.jpg".into()),
        playlist: "https://stream.place/playback/full.m3u8".into(),
        aspect_ratio: Some(AspectRatio {
            width: 16,
            height: 9,
        }),
        source: VideoSource::Streamplace,
        created_at: "2026-01-02T03:04:05.000Z".into(),
        like_count: 7,
        live: true,
        viewer_count: Some(42),
        duration_ms: Some(5_400_000),
        activity: Some("music".into()),
        // no upstream carries captions yet, but the full instance pins
        // CaptionTrack's wire shape so it cannot drift unnoticed
        captions: vec![CaptionTrack {
            src: "https://cdn.example.com/full.vtt".into(),
            lang: "en".into(),
            label: "english".into(),
        }],
        blur: Some(Blur {
            label: "!warn".into(),
        }),
    });
    let video_bare = Brick::Video(VideoBrick {
        id: "at://did:plc:bare/app.bsky.feed.post/vid".into(),
        url: "https://bsky.app/profile/bare.example.com/post/vid".into(),
        author: author_bare(),
        title: "".into(),
        poster: None,
        playlist: "https://video.example.com/bare.m3u8".into(),
        aspect_ratio: None,
        source: VideoSource::Bluesky,
        created_at: "2026-01-02T03:04:05.000Z".into(),
        like_count: 0,
        live: false,
        viewer_count: None,
        duration_ms: None,
        activity: None,
        // empty on purpose: the bare instance pins skip-when-empty absence
        captions: Vec::new(),
        blur: None,
    });

    [
        (post_full, "full"),
        (post_bare, "bare"),
        (blog_full, "full"),
        (blog_bare, "bare"),
        (video_full, "full"),
        (video_bare, "bare"),
    ]
    .into_iter()
    .map(|(brick, shape)| {
        let value = serde_json::to_value(&brick).expect("a brick serializes");
        let key = kind_key(&brick);
        assert_eq!(
            value["kind"], key,
            "the serialized kind tag must equal the fixture key, or the web \
             side would check the wrong literal"
        );
        (brick, shape, value)
    })
    .collect()
}

/// Every error code the fixture must cover, in one place. Same construction
/// as `ALL_KINDS`: `code_key` indexes into it, and `contract()` asserts the
/// fixture's error map carries exactly these keys.
const ALL_CODES: [&str; 4] = [
    "bad_request",
    "actor_not_found",
    "login_required",
    "upstream",
];

/// One canonical instance per AppError variant. Kept a Vec on purpose: the
/// length is checked against `ALL_CODES` in `contract()`, not by the type.
fn errors() -> Vec<AppError> {
    vec![
        AppError::BadRequest("actor"),
        AppError::ActorNotFound("nobody.example.com".into()),
        AppError::LoginRequired("sealed.example.com".into()),
        AppError::Upstream("appview timed out".into()),
    ]
}

/// The fixture key for one error. The forcing chain for a new AppError
/// variant mirrors `kind_key`: a new match arm must index a new `ALL_CODES`
/// slot (an out-of-bounds constant index fails the build, so the array must
/// grow), and the key-set assert in `contract()` then fails until `errors()`
/// carries an instance of the new variant. The key must also equal what the
/// engine actually puts on the wire, asserted here against `status_and_code`.
fn code_key(error: &AppError) -> &'static str {
    let code = match error {
        AppError::BadRequest(_) => ALL_CODES[0],
        AppError::ActorNotFound(_) => ALL_CODES[1],
        AppError::LoginRequired(_) => ALL_CODES[2],
        AppError::Upstream(_) => ALL_CODES[3],
    };
    assert_eq!(
        code,
        error.status_and_code().1,
        "the fixture key must equal the wire code, or the web side would \
         check the wrong literal"
    );
    code
}

/// Assemble the whole contract document. Object keys carry the literal
/// vocabulary; values carry the serde output of real model instances.
fn contract() -> Value {
    let mut brick_map = serde_json::Map::new();
    let mut committed_items: Vec<Brick> = Vec::new();
    for (brick, shape, value) in bricks() {
        let entry = brick_map
            .entry(kind_key(&brick))
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        entry
            .as_object_mut()
            .expect("a per-kind entry is an object")
            .insert(shape.to_string(), value);
        committed_items.push(brick);
    }

    // the three FeedResponse shapes the engine emits: a committed page, a
    // warming preview (the only place `warming` appears), and the final page
    // (cursor exhausted). Built from the real struct so serde's skip rules,
    // not this test, decide what appears.
    let committed = FeedResponse {
        items: committed_items,
        cursor: Some("opaque-cursor-token".into()),
        warming: None,
    };
    let preview = FeedResponse {
        items: vec![],
        cursor: Some("opaque-cursor-token".into()),
        warming: Some(true),
    };
    let final_page = FeedResponse {
        items: vec![],
        cursor: None,
        warming: None,
    };

    // coverage by construction: the fixture must carry exactly one brick map
    // entry per ALL_KINDS token. This is the assert the kind_key forcing
    // chain lands on: a grown ALL_KINDS fails here until bricks() catches up.
    assert_eq!(
        brick_map.keys().map(String::as_str).collect::<Vec<_>>(),
        {
            let mut sorted = ALL_KINDS;
            sorted.sort_unstable();
            sorted
        },
        "the fixture must carry canonical instances of every Brick kind"
    );

    // every error code, in both wire shapes: the server body (status on the
    // response line, absent from the body) and the wasm throw (status in-band)
    let mut error_map = serde_json::Map::new();
    for error in errors() {
        let code = code_key(&error);
        error_map.insert(
            code.to_string(),
            json!({
                "server": serde_json::to_value(error.envelope()).expect("envelope serializes"),
                "wasm": serde_json::to_value(error.envelope_with_status())
                    .expect("envelope serializes"),
            }),
        );
    }
    // the code_key forcing chain lands here, exactly like the brick one above
    assert_eq!(
        error_map.keys().map(String::as_str).collect::<Vec<_>>(),
        {
            let mut sorted = ALL_CODES;
            sorted.sort_unstable();
            sorted
        },
        "the fixture must carry both envelopes of every AppError code"
    );

    // the query vocabulary: `?mode=` names only glaze (anything else is the
    // default full wall) and `?intent=` names preview and freeze (absent is a
    // normal committed page). Each token is bound ONCE and used for both the
    // parser assert and the fixture key, so a one-sided rename cannot stay
    // green: the parser assert fails on a Rust rename, and the changed key
    // fails tsc on the web side.
    const GLAZE: &str = "glaze";
    const PREVIEW: &str = "preview";
    const FREEZE: &str = "freeze";
    assert_eq!(Mode::from_query(Some(GLAZE)), Mode::Glaze);
    assert_eq!(Mode::from_query(None), Mode::Wall);
    assert_eq!(FeedIntent::from_query(Some(PREVIEW)), FeedIntent::Preview);
    assert_eq!(FeedIntent::from_query(Some(FREEZE)), FeedIntent::Freeze);
    assert_eq!(FeedIntent::from_query(None), FeedIntent::Normal);
    let mut mode_map = serde_json::Map::new();
    mode_map.insert(GLAZE.to_string(), Value::Bool(true));
    let mut intent_map = serde_json::Map::new();
    for intent in [PREVIEW, FREEZE] {
        intent_map.insert(intent.to_string(), Value::Bool(true));
    }
    let query = json!({
        "mode": Value::Object(mode_map),
        "intent": Value::Object(intent_map),
    });

    // enum string values that ride INSIDE bricks, keyed so keyof sees them
    let mut source_map = serde_json::Map::new();
    for source in [VideoSource::Bluesky, VideoSource::Streamplace] {
        let tag = serde_json::to_value(source).expect("a video source serializes");
        let tag = tag
            .as_str()
            .expect("a video source is a string")
            .to_string();
        source_map.insert(tag, Value::Bool(true));
    }

    json!({
        "bricks": Value::Object(brick_map),
        "pages": {
            "committed": serde_json::to_value(&committed).expect("a page serializes"),
            "preview": serde_json::to_value(&preview).expect("a page serializes"),
            "final": serde_json::to_value(&final_page).expect("a page serializes"),
        },
        "errors": Value::Object(error_map),
        "query": query,
        "vocab": { "videoSource": Value::Object(source_map) },
    })
}

#[test]
fn wire_contract_matches_the_committed_fixture() {
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&contract()).expect("the contract document serializes")
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract.json");

    if std::env::var_os("UPDATE_FIXTURE").is_some() {
        std::fs::write(&path, rendered).expect("the fixture is writable");
        return;
    }

    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        rendered, committed,
        "\nmortar's serialization no longer matches tests/fixtures/contract.json.\n\
         If this wire change is intentional, regenerate the fixture:\n\n    \
         UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract\n\n\
         then run `pnpm check:ci` in web/ so web/src/lib/contract-check.ts can\n\
         confirm web/src/lib/types.ts still matches, and commit both files."
    );
}
