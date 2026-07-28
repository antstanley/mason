import { browser } from "$app/environment";
import { httpUrl } from "$lib/url";

/** Atmosphere clients that mirror bsky.app's URL structure, so opening a post
 *  in one of them is a host swap and nothing more. Anyone who lives in an
 *  alternative client should not be bounced into an app they do not use. */
export const CLIENTS = [
  { id: "bsky.app", label: "Bluesky", host: "bsky.app" },
  { id: "mu.social", label: "Mu Social", host: "mu.social" },
  { id: "blacksky.community", label: "Blacksky", host: "blacksky.community" },
  { id: "twinkl.social", label: "Twinkl", host: "twinkl.social" },
  { id: "witchsky.app", label: "Witchsky", host: "witchsky.app" },
] as const;

export type ClientId = (typeof CLIENTS)[number]["id"];

const STORAGE_KEY = "mason:client";
const DEFAULT: ClientId = "bsky.app";

function stored(): ClientId {
  if (!browser) return DEFAULT;
  const saved = localStorage.getItem(STORAGE_KEY);
  return CLIENTS.some((c) => c.id === saved) ? (saved as ClientId) : DEFAULT;
}

class ClientState {
  id = $state<ClientId>(stored());

  set(id: ClientId) {
    this.id = id;
    if (browser) localStorage.setItem(STORAGE_KEY, id);
  }

  get host(): string {
    return CLIENTS.find((c) => c.id === this.id)?.host ?? DEFAULT;
  }
}

/** What to call the place a link is about to open, for a control that names its
 *  destination rather than saying "open the post".
 *
 *  Keyed on the FINISHED url rather than on `client.id`, because those two
 *  disagree: `clientUrl` rewrites bsky.app links and passes everything else
 *  through untouched, so a stream.place link on a Twinkl setting still goes to
 *  stream.place. Reading the label off the setting would promise a client the
 *  reader is not about to land in. `null` when the host belongs to no client on
 *  the list, which is the caller's cue to say something generic. */
export function clientName(url: string): string | null {
  let host: string;
  try {
    host = new URL(url).hostname;
  } catch {
    return null;
  }
  return CLIENTS.find((c) => c.host === host)?.label ?? null;
}

export const client = new ClientState();

/** How each client spells the route bsky.app calls `/profile/`.
 *
 *  Most mirror bsky.app exactly, so swapping the host is the whole rewrite.
 *  Twinkl does not: it serves a profile at `/@<handle>` and a post at
 *  `/@<handle>/post/<rkey>`, so a host swap alone would hand somebody a 404.
 *  Verified against the live sites on 2026-07-27 rather than assumed.
 *
 *  A Record keyed by the union rather than an optional field, so a client added
 *  to CLIENTS is a compile error here until somebody has checked which shape it
 *  uses. Getting this wrong is silent: the link still opens, it just lands
 *  nowhere. */
const PROFILE_PREFIX: Record<ClientId, string> = {
  "bsky.app": "/profile/",
  "mu.social": "/profile/",
  "blacksky.community": "/profile/",
  "twinkl.social": "/@",
  "witchsky.app": "/profile/",
};

const BSKY_PROFILE = "/profile/";

/** Rewrite a bsky.app link to the reader's chosen client. Only bsky.app is
 *  rewritten: blog links and stream.place pages are not Bluesky posts, and no
 *  other client knows how to show them. They pass through untouched.
 *
 *  It is for a url MASON built, from a brick's own `url` field, which is always
 *  a profile or a post. A url somebody else wrote is a different question, and
 *  takes `httpUrl` on its own: see the reader's external embed
 *  (BrickReader.svelte). */
export function clientUrl(url: string, host: string = client.host): string {
  // the scheme guard, in the one spelling lib/url.ts holds: what may not reach
  // an href from a link card may not reach one from here either
  const safe = httpUrl(url);
  // nowhere to go, or nowhere else to go
  if (!safe || host === "bsky.app") return safe;
  // parsed a second time rather than threaded out of the guard: it cannot throw
  // here, because `httpUrl` only answers with a string it parsed itself, and a
  // guard handing back a `URL` would make every other caller reserialize a link
  // it only asked to have vetted.
  const parsed = new URL(safe);
  // only bsky.app posts are rewritten; everything else passes through untouched
  if (parsed.hostname !== "bsky.app") return safe;
  parsed.hostname = host;
  // and the path, for a client that spells the profile route differently. The
  // rest of the path rides along unchanged: every one of these clients agrees
  // about `/post/<rkey>` after the handle.
  const prefix = CLIENTS.find((c) => c.host === host)?.id;
  const spelling = prefix === undefined ? BSKY_PROFILE : PROFILE_PREFIX[prefix];
  if (spelling !== BSKY_PROFILE && parsed.pathname.startsWith(BSKY_PROFILE)) {
    parsed.pathname = spelling + parsed.pathname.slice(BSKY_PROFILE.length);
  }
  return parsed.toString();
}
