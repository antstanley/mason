import { browser } from "$app/environment";
import { APPVIEW } from "$lib/appview";

/** The half of `app.bsky.feed.getFeedGenerator`'s answer the header reads.
 *  Everything is optional because it is somebody else's record: a generator can
 *  be published with no display name and no avatar at all. */
interface FeedGeneratorResponse {
  view?: {
    displayName?: string;
    avatar?: string;
    creator?: { handle?: string };
  };
}

/** The generator's rkey, which is the last path segment of both spellings
 *  mortar accepts (`at://<did>/app.bsky.feed.generator/<rkey>` and
 *  `https://bsky.app/profile/<actor>/feed/<rkey>`).
 *
 *  It is the header's name for a feed until the AppView answers, and its name
 *  for good if the AppView never does. Ugly, but it is the only part of a raw
 *  reference a reader recognises, and a header that waited for the network
 *  would show an empty button on every feed wall's first paint. A reference
 *  with no slash at all, or one ending in one, keeps the whole string rather
 *  than naming the feed nothing. */
function feedRkey(raw: string): string {
  const rkey = raw.slice(raw.lastIndexOf("/") + 1);
  return rkey || raw;
}

/** Exported for the unit tests, which build throwaway instances; the app only
 *  ever uses the `feedInfo` singleton below. */
export class FeedInfoState {
  /** the reference we last loaded, so a re-render doesn't refetch */
  private loaded: string | null = null;
  avatar = $state<string | null>(null);
  /** What the header calls this feed: the generator's display name once the
   *  AppView answers, the rkey before that and if it never does. */
  name = $state("");
  /** The handle that published the feed. Display names are not unique (two
   *  "Discover" feeds are ordinary), so this is what tells a screen-reader
   *  reader which one they are on. */
  creator = $state<string | null>(null);

  /** Name the feed in the header, and never block on it.
   *
   *  This exists for the same reason `profile` does: a feed response carries
   *  bricks, not the identity of the algorithm that ranked them, and putting
   *  that identity on the wire would mean a field present on some walls and
   *  null on others to carry two strings the header alone consumes. So the
   *  header asks the public AppView itself, exactly as it asks `getProfile`
   *  for a wall owner's face. */
  load(feed: string) {
    if (feed === this.loaded) return;
    this.loaded = feed;
    this.avatar = null;
    this.creator = null;
    // set before the guard and before the request, so the header has something
    // to say on the first frame of every feed wall, including in the build with
    // no browser at all
    this.name = feedRkey(feed);
    if (!browser || !feed) return;

    void fetch(`${APPVIEW}/xrpc/app.bsky.feed.getFeedGenerator?feed=${encodeURIComponent(feed)}`)
      .then((res) => (res.ok ? res.json() : null))
      .then((data: FeedGeneratorResponse | null) => {
        // ignore a response that arrived after the reference changed under us
        if (this.loaded !== feed || !data?.view) return;
        // a generator published without a display name keeps the rkey; an empty
        // button names the feed less than the ugly string does
        this.name = data.view.displayName || this.name;
        this.avatar = data.view.avatar ?? null;
        this.creator = data.view.creator?.handle ?? null;
      })
      .catch(() => {
        // no name and no face: the header keeps the rkey, and the wall behind
        // it is unaffected. A feed mason can lay is a feed mason lays whether
        // or not it can say whose it is.
      });
  }
}

export const feedInfo = new FeedInfoState();
