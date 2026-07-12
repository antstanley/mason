import { browser } from "$app/environment";

/** Atmosphere clients that mirror bsky.app's URL structure, so opening a post
 *  in one of them is a host swap and nothing more. Anyone who lives in an
 *  alternative client should not be bounced into an app they do not use. */
export const CLIENTS = [
  { id: "bsky.app", label: "Bluesky", host: "bsky.app" },
  { id: "mu.social", label: "mu.social", host: "mu.social" },
  { id: "blacksky.community", label: "Blacksky", host: "blacksky.community" },
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

/** Rewrite a bsky.app link to the reader's chosen client. Only bsky.app is
 *  rewritten: blog links and Steam pages are not atproto posts and must be
 *  left exactly as they are. */
export function clientUrl(url: string, host: string = client.host): string {
  if (host === "bsky.app") return url;
  try {
    const parsed = new URL(url);
    if (parsed.hostname !== "bsky.app") return url;
    parsed.hostname = host;
    return parsed.toString();
  } catch {
    return url;
  }
}
