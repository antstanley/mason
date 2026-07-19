// Mirrors mortar's serde output (server/crates/mortar-core/src/model.rs).
//
// Drift guard: web/src/lib/contract-check.ts asserts these types against
// server/crates/mortar-core/tests/fixtures/contract.json, which a Rust test
// (server/crates/mortar-core/tests/contract.rs) pins byte-for-byte against the
// real serialization. A rename or shape change on either side now fails a
// check (cargo test there, tsc here) instead of shipping silently.
//
// The import below is the guard's own tripwire: it is type-only (erased at
// compile time, so nothing reaches the runtime graph; the cycle with
// contract-check importing this file back is legal for types), and it exists
// so that DELETING contract-check.ts is itself a tsc error rather than a
// silent loss of coverage. The named block is empty on purpose: only the
// module's existence is being asserted, none of its exports are wanted here.
// oxlint-disable-next-line no-empty-named-blocks
import type {} from "./contract-check";

export interface Author {
  did: string;
  handle: string;
  displayName: string | null;
  avatar: string | null;
}

interface AspectRatio {
  width: number;
  height: number;
}

interface ImageEmbed {
  src: string;
  alt: string;
  aspectRatio: AspectRatio | null;
}

interface ExternalEmbed {
  uri: string;
  title: string;
  description: string;
  thumb: string | null;
}

/** Present when a `!warn` label covers a brick's media behind a reveal. Absent
 *  (not null) when there is nothing to cover, since mortar skips serializing it. */
export interface Blur {
  label: string;
}

export interface PostBrick {
  kind: "post";
  id: string;
  url: string;
  author: Author;
  text: string;
  createdAt: string;
  likeCount: number;
  repostCount: number;
  images: ImageEmbed[];
  external: ExternalEmbed | null;
  blur?: Blur;
}

interface Publication {
  name: string;
  url: string;
  icon: string | null;
}

export interface BlogBrick {
  kind: "blog";
  id: string;
  url: string;
  author: Author;
  title: string;
  description: string | null;
  coverImage: string | null;
  publication: Publication;
  tags: string[];
  publishedAt: string;
}

export interface VideoBrick {
  kind: "video";
  id: string;
  url: string;
  author: Author;
  title: string;
  poster: string | null;
  playlist: string;
  aspectRatio: AspectRatio | null;
  source: "bluesky" | "streamplace";
  createdAt: string;
  likeCount: number;
  /** Streamplace only: this stream is happening right now. */
  live: boolean;
  viewerCount: number | null;
  durationMs: number | null;
  /** What the streamer says they are doing ("music", a game, ...). */
  activity: string | null;
  blur?: Blur;
}

export type Brick = PostBrick | BlogBrick | VideoBrick;

export interface FeedResponse {
  items: Brick[];
  cursor: string | null;
  /** Only present on a preview response: whether the wall is still warming, so
   *  the client keeps polling and reflowing the first screen until it settles. */
  warming?: boolean;
}

/** The named walls a feed request can ask for via `?mode=`; absent asks for
 *  the default full wall. Mirrors Mode::from_query in
 *  server/crates/mortar-core/src/mode.rs; pinned by the contract fixture. */
export type FeedMode = "glaze";

/** The machine codes mortar itself can emit in an ErrorEnvelope. Mirrors
 *  AppError::status_and_code in server/crates/mortar-core/src/error.rs; pinned
 *  by the contract fixture. The web adds its own out-of-band codes ("wasm",
 *  "unknown") for failures that never reached mortar, so ErrorEnvelope.error
 *  stays a plain string. */
export type MortarErrorCode = "bad_request" | "actor_not_found" | "login_required" | "upstream";

/** The one error shape mortar emits in both build modes: the native server
 *  sends it as a JSON body (status on the response line), the wasm build
 *  throws it as a JSON string with `status` in-band. Mirrors ErrorEnvelope in
 *  server/crates/mortar-core/src/error.rs, where a fixture test pins the exact
 *  wire strings per error variant. */
export interface ErrorEnvelope {
  /** Machine code ("actor_not_found", "login_required", ...); classification
   *  happens on this, so codes are wire contract, not cosmetics. */
  error: string;
  /** Human-readable detail; display only, never matched on. */
  message: string;
  /** HTTP status; present only on the wasm channel, which has no HTTP layer
   *  of its own until the service worker builds the Response. */
  status?: number;
}
