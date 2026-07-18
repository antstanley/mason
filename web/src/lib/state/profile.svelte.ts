import { browser } from "$app/environment";

/** The wall owner's face for the header. The feed never carries the actor's own
 *  profile (their bricks may not even appear on their wall), so we ask the
 *  public AppView directly, the same surface local mode already talks to. A
 *  miss (bad handle, offline, the demo fixture) leaves `avatar` null and the
 *  switch button falls back to an initial. */
const APPVIEW = "https://public.api.bsky.app";

/** The reserved self-label an account sets to opt out of logged-out views. */
const NO_UNAUTHENTICATED = "!no-unauthenticated";

class ProfileState {
  /** the handle we last loaded, so a re-render doesn't refetch */
  private loaded: string | null = null;
  avatar = $state<string | null>(null);
  /** the owner asked to be seen only by signed-in visitors: the engine seals
   *  the wall, and the header withholds their face to match */
  optedOut = $state(false);

  load(actor: string) {
    if (actor === this.loaded) return;
    this.loaded = actor;
    this.avatar = null;
    this.optedOut = false;
    if (!browser || !actor) return;

    // the demo wall is a fixture with no real profile; give it a stable face
    // from the same source the fixture bricks use
    if (actor === "demo") {
      this.avatar = "https://picsum.photos/seed/masondemo/96/96";
      return;
    }

    void fetch(`${APPVIEW}/xrpc/app.bsky.actor.getProfile?actor=${encodeURIComponent(actor)}`)
      .then((res) => (res.ok ? res.json() : null))
      .then((data: { avatar?: string; labels?: { val: string }[] } | null) => {
        // ignore a response that arrived after the handle changed under us
        if (this.loaded !== actor || !data) return;
        this.optedOut = data.labels?.some((l) => l.val === NO_UNAUTHENTICATED) ?? false;
        // an opted-out owner shows no face here; the wall behind is sealed
        this.avatar = this.optedOut ? null : (data.avatar ?? null);
      })
      .catch(() => {
        // no face; the button shows an initial instead
      });
  }
}

export const profile = new ProfileState();
