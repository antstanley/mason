/** The public Bluesky AppView, read straight from the browser.
 *
 *  Chrome asks it things the wall's own response cannot carry: the wall owner's
 *  face, the feed generator's name. Those are identity for the header, not
 *  content for the wall, so they never touch mortar; content moderation stays
 *  where it belongs, on the bricks mortar lays.
 *
 *  It lives in one module because there is more than one reader of it now, and
 *  a base URL copied per reader is a base URL that drifts per reader. The
 *  engine keeps its own copy (`appview_base` in
 *  server/crates/mortar-core/src/config.rs) deliberately: that one is
 *  configurable per deployment and this one is the browser's, and collapsing
 *  them would mean shipping engine config to the client to name a face. */
export const APPVIEW = "https://public.api.bsky.app";
