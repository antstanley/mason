//! M0 fixture bricks; a deterministic wall of fake content in all three
//! kinds, so the whole pipe (cursor paging included) works before any real
//! upstream API is wired in. Replaced by real sources from M1 onward.

use crate::model::*;

const POST_TEXTS: &[&str] = &[
    "just shipped a new feed algorithm and honestly? the bricks are laying themselves",
    "hot take: masonry layouts are just newspapers that learned to scroll",
    "day 47 of building in public. the wall grows.",
    "if your discovery feed doesn't spark joy, throw the whole algorithm out",
    "the mortar between good posts is other good posts",
    "TIL you can livestream over atproto now. everything is a wall if you squint",
    "atproto is the most fun I've had on the internet since 2009",
    "my follow graph is 40% shitposters, 40% blog nerds, 20% streamers. perfect balance",
    "unpopular opinion: endless scroll is fine when the content is actually good",
    "publish on your own site. syndicate everywhere. the dream is alive",
    "every timeline is a wall. every post is a brick. every like is a trowel of mortar",
    "brb rewriting my entire blog in a lexicon three people have heard of (it rules)",
];

const BLOG_TITLES: &[(&str, &str)] = &[
    (
        "Why I moved my blog to the Atmosphere",
        "Owning your words means owning the records too. A migration story with surprisingly few regrets.",
    ),
    (
        "Masonry layouts: a love letter",
        "Fifteen years of CSS hacks, and we still measure column heights by hand. Here's why that's okay.",
    ),
    (
        "The quiet joy of small protocols",
        "Not everything needs to be a platform. Sometimes a lexicon and a dream is enough.",
    ),
    (
        "Building a feed algorithm nobody hates",
        "Recency, diversity, and a little seeded chaos. What I learned mixing three content types into one wall.",
    ),
    (
        "Notes on digital gardening",
        "Blogs are back, but weirder this time. A tour of the new indie publishing stack.",
    ),
    (
        "HLS everywhere: how video quietly standardized",
        "From livestreams to social clips, everything is an m3u8 now.",
    ),
];

const STREAM_TITLES: &[&str] = &[
    "laying bricks live, no plan, no mercy",
    "kiln maintenance and chill",
    "reading the atproto spec out loud (part 4)",
    "late night mortar mixing",
];

const HANDLES: &[(&str, &str)] = &[
    ("bricklayer.example.com", "Brick Layer"),
    ("mortarmaid.example.com", "Mortar Maid"),
    ("kilnfired.example.com", "Kiln Fired"),
    ("groutful.example.com", "Groutful Dead"),
    ("trowelpunk.example.com", "Trowel Punk"),
    ("plastercaster.example.com", "Plaster Caster"),
];

fn author(i: usize) -> Author {
    let (handle, name) = HANDLES[i % HANDLES.len()];
    Author {
        did: format!("did:plc:fixture{i}"),
        handle: handle.into(),
        display_name: Some(name.into()),
        avatar: Some(format!(
            "https://picsum.photos/seed/avatar{}/96/96",
            i % HANDLES.len()
        )),
    }
}

fn created_at(i: usize) -> String {
    // Deterministic timestamps marching backwards from a fixed anchor
    format!(
        "2026-07-{:02}T{:02}:{:02}:00Z",
        10 - (i / 24).min(9),
        23 - (i % 24),
        (i * 7) % 60
    )
}

/// The full fixture pool: 120 bricks, roughly 70/15/15 post/blog/video.
pub fn pool() -> Vec<Brick> {
    (0..120).map(brick).collect()
}

fn brick(i: usize) -> Brick {
    match i % 20 {
        // 3 blogs per 20
        3 | 10 | 17 => {
            let (title, desc) = BLOG_TITLES[i % BLOG_TITLES.len()];
            Brick::Blog(BlogBrick {
                id: format!("fixture-blog-{i}"),
                url: format!("https://example.com/blog/{i}"),
                author: author(i),
                title: title.into(),
                description: Some(desc.into()),
                cover_image: (i % 40 != 10)
                    .then(|| format!("https://picsum.photos/seed/cover{i}/800/500")),
                publication: Publication {
                    name: "The Daily Brick".into(),
                    url: "https://example.com/blog".into(),
                    icon: None,
                },
                tags: vec!["atproto".into(), "indieweb".into()],
                published_at: created_at(i),
            })
        }
        // 3 videos per 20: alternate Bluesky clips and Streamplace streams,
        // one of which is live (brick 6 of the first twenty, so the demo wall
        // opens on it just as a real one would)
        6 | 13 | 19 => {
            let streamplace = i.is_multiple_of(2);
            let live = i == 6;
            Brick::Video(VideoBrick {
                id: format!("fixture-video-{i}"),
                url: format!("https://example.com/video/{i}"),
                author: author(i),
                title: if streamplace {
                    STREAM_TITLES[i % STREAM_TITLES.len()].into()
                } else {
                    POST_TEXTS[i % POST_TEXTS.len()].into()
                },
                poster: Some(format!("https://picsum.photos/seed/poster{i}/800/450")),
                playlist: "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".into(),
                aspect_ratio: Some(AspectRatio {
                    width: 16,
                    height: 9,
                }),
                source: if streamplace {
                    VideoSource::Streamplace
                } else {
                    VideoSource::Bluesky
                },
                created_at: created_at(i),
                like_count: (i as u64 * 13) % 500,
                live,
                viewer_count: live.then_some(37),
                duration_ms: (streamplace && !live).then_some(4_920_000),
                activity: streamplace.then(|| "bricklaying".into()),
                captions: Vec::new(),
                blur: None,
            })
        }
        // everything else: posts, some with images
        _ => {
            let with_image = i.is_multiple_of(3);
            Brick::Post(PostBrick {
                id: format!("fixture-post-{i}"),
                url: format!("https://bsky.app/profile/fixture/post/{i}"),
                author: author(i),
                text: POST_TEXTS[i % POST_TEXTS.len()].into(),
                created_at: created_at(i),
                like_count: (i as u64 * 31) % 900,
                repost_count: (i as u64 * 7) % 120,
                images: if with_image {
                    let h = [500, 700, 900, 620][i % 4];
                    // vary the count so the glaze wall shows every layout:
                    // single, the 2- and 3-up grids, and the 4+ carousel
                    let n = match (i / 3) % 4 {
                        1 => 2,
                        2 => 3,
                        3 => 5,
                        _ => 1,
                    };
                    (0..n)
                        .map(|k| ImageEmbed {
                            src: format!("https://picsum.photos/seed/img{i}-{k}/800/{h}"),
                            alt: format!("fixture image {} of {n}", k + 1),
                            aspect_ratio: Some(AspectRatio {
                                width: 800,
                                height: h,
                            }),
                        })
                        .collect()
                } else {
                    vec![]
                },
                external: None,
                blur: None,
            })
        }
    }
}
