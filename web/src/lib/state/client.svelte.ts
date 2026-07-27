import { browser } from "$app/environment";

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
 *  other client knows how to show them. They pass through untouched. */
export function clientUrl(url: string, host: string = client.host): string {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return "";
  }
  // only http(s) may reach an <a href>; javascript:/data:/vbscript: are dropped
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return "";
  // only bsky.app posts are rewritten; everything else passes through untouched
  if (host === "bsky.app" || parsed.hostname !== "bsky.app") return url;
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
