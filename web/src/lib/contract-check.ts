// The TypeScript half of the wire-contract drift guard (issue #33).
//
// types.ts hand-mirrors mortar's serde output, and this file is what makes
// that mirroring checked instead of hoped-for. It imports the contract
// fixture (server/crates/mortar-core/tests/fixtures/contract.json), which a
// Rust test (tests/contract.rs in mortar-core) pins byte-for-byte against the
// engine's real serialization, and asserts it against the types in types.ts
// and api.ts. A rename, a missing field, a wrong casing, or a changed literal
// on either side fails `pnpm check:ci` here or `cargo test` there.
//
// tsc-only, never bundled: nothing in the app imports this module, so the
// fixture never reaches the runtime import graph. It exists purely to be
// typechecked (and is wired as a knip entry for exactly that reason).
//
// Two mechanisms, because tsc widens JSON string VALUES to `string`:
// - structure (field names, optionality, null-vs-absent) is checked with
//   `satisfies` against Wire<T>, the types with literal strings relaxed;
// - vocabulary (brick kinds, error codes, video sources, query tokens) rides
//   as fixture object KEYS, which stay literal, and is compared with Equal<>
//   in both directions.
//
// After an intentional wire change on the Rust side, regenerate with:
//   UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract
// then update types.ts until this file typechecks again.

import contract from "../../../server/crates/mortar-core/tests/fixtures/contract.json";
import type { FeedIntent } from "./api";
import type {
  Author,
  BlogBrick,
  Blur,
  Brick,
  ErrorEnvelope,
  FeedMode,
  FeedResponse,
  MortarErrorCode,
  PostBrick,
  VideoBrick,
} from "./types";

/** T with every literal string relaxed to `string`, matching how tsc types an
 *  imported JSON file. Structure, optionality and nullability survive intact;
 *  the literals themselves are checked separately via fixture keys. */
type Wire<T> = T extends string
  ? string
  : T extends readonly (infer U)[]
    ? Wire<U>[]
    : T extends object
      ? { [K in keyof T]: Wire<T[K]> }
      : T;

/** Structural checks: every canonical instance in the fixture must fit the
 *  corresponding TS type. `full` instances carry every optional field, `bare`
 *  ones none, so both sides of optional-vs-nullable modeling are exercised. */
export const structure = {
  post: contract.bricks.post satisfies Record<"full" | "bare", Wire<PostBrick>>,
  blog: contract.bricks.blog satisfies Record<"full" | "bare", Wire<BlogBrick>>,
  video: contract.bricks.video satisfies Record<"full" | "bare", Wire<VideoBrick>>,
  // a committed page (mixed items, cursor, no warming), a warming preview
  // (the only response that carries `warming`), and the final page (cursor
  // exhausted)
  pages: contract.pages satisfies Record<"committed" | "preview" | "final", Wire<FeedResponse>>,
  // per error code, both wire shapes: the server body (status on the response
  // line) and the wasm throw (status in-band)
  errors: contract.errors satisfies Record<
    MortarErrorCode,
    { server: Wire<ErrorEnvelope>; wasm: Wire<ErrorEnvelope> }
  >,
};

type Equal<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;
type Assert<T extends true> = T;

// --- vocabulary: fixture keys vs TS literal unions, both directions ---------

export type BrickKindsMatch = Assert<Equal<keyof typeof contract.bricks, Brick["kind"]>>;
export type ErrorCodesMatch = Assert<Equal<keyof typeof contract.errors, MortarErrorCode>>;
export type IntentVocabularyMatches = Assert<Equal<keyof typeof contract.query.intent, FeedIntent>>;
export type ModeVocabularyMatches = Assert<Equal<keyof typeof contract.query.mode, FeedMode>>;
export type VideoSourcesMatch = Assert<
  Equal<keyof typeof contract.vocab.videoSource, VideoBrick["source"]>
>;

// --- field sets: a `full` instance carries EVERY field, so its exact key set
// must equal the interface's. This is what catches a field mortar gained that
// types.ts does not know yet, and renames of optional fields, in both
// directions. -----------------------------------------------------------------

type PostFull = typeof contract.bricks.post.full;
export type PostFieldsMatch = Assert<Equal<keyof PostFull, keyof PostBrick>>;
export type AuthorFieldsMatch = Assert<Equal<keyof PostFull["author"], keyof Author>>;
export type BlurFieldsMatch = Assert<Equal<keyof PostFull["blur"], keyof Blur>>;
export type ImageFieldsMatch = Assert<
  Equal<keyof PostFull["images"][number], keyof PostBrick["images"][number]>
>;
export type AspectRatioFieldsMatch = Assert<
  Equal<
    keyof PostFull["images"][number]["aspectRatio"],
    keyof NonNullable<PostBrick["images"][number]["aspectRatio"]>
  >
>;
export type ExternalFieldsMatch = Assert<
  Equal<keyof PostFull["external"], keyof NonNullable<PostBrick["external"]>>
>;
export type BlogFieldsMatch = Assert<
  Equal<keyof typeof contract.bricks.blog.full, keyof BlogBrick>
>;
export type PublicationFieldsMatch = Assert<
  Equal<keyof typeof contract.bricks.blog.full.publication, keyof BlogBrick["publication"]>
>;
export type VideoFieldsMatch = Assert<
  Equal<keyof typeof contract.bricks.video.full, keyof VideoBrick>
>;
export type FeedResponseFieldsMatch = Assert<
  Equal<keyof typeof contract.pages.preview, keyof FeedResponse>
>;
export type WasmEnvelopeFieldsMatch = Assert<
  Equal<keyof typeof contract.errors.bad_request.wasm, keyof ErrorEnvelope>
>;
// the server body omits `status` (it rides on the HTTP response line instead)
export type ServerEnvelopeFieldsMatch = Assert<
  Equal<keyof typeof contract.errors.bad_request.server, Exclude<keyof ErrorEnvelope, "status">>
>;
