// The one scheme guard on the web side of the wire.
//
// It lives in a plain `.ts` module rather than beside its first caller because
// this file is typechecked and unit tested and a `.svelte` body is neither, and
// because two places need the same answer: `clientUrl` (state/client.svelte.ts),
// which vets a link before it decides whether to rewrite it, and the brick
// reader's external embed, which must be vetted and NOT rewritten. Two ad hoc
// spellings of "only http(s)" is one spelling that can quietly stop matching.

/** A url that may be handed to an `<a href>`, or `""` when it may not.
 *
 *  Only `http:` and `https:` survive. `javascript:`, `data:` and `vbscript:` are
 *  a script mason would run on its own origin the moment somebody touched the
 *  link, and every url on the wall is a string a stranger wrote into their own
 *  record. mortar vets each one at the source (`is_http_url` in
 *  `sources/util.rs`), so this is the second line rather than the rule: in
 *  server mode the SPA talks to a native mortar it was not built alongside, and
 *  a brick from an older one must not be able to put a scheme like that on an
 *  href.
 *
 *  The string comes back exactly as it arrived rather than reserialized through
 *  `URL`, because a link card prints its own address under itself: an href that
 *  says one thing while the words beneath it say another is a broken promise
 *  whichever way round the difference runs.
 *
 *  `""` rather than null, so a caller can write `href={httpUrl(x) || undefined}`
 *  and mean it: `href=""` resolves to the current document, which is a link that
 *  quietly reopens mason, where no href at all is not a link at all and takes
 *  neither a click nor a Tab. */
export function httpUrl(url: string): string {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    // not a url at all: a relative string, an empty field, somebody's prose
    return "";
  }
  return parsed.protocol === "http:" || parsed.protocol === "https:" ? url : "";
}
