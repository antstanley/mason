import { browser } from "$app/environment";

/** The wall owner's face for the header. The feed never carries the actor's own
 *  profile (their bricks may not even appear on their wall), so we ask the
 *  public AppView directly, the same surface local mode already talks to. A
 *  miss (bad handle, offline, the demo fixture) leaves `avatar` null and the
 *  switch button falls back to an initial. */
const APPVIEW = "https://public.api.bsky.app";

class ProfileState {
  /** the handle we last loaded, so a re-render doesn't refetch */
  private loaded: string | null = null;
  avatar = $state<string | null>(null);

  load(actor: string) {
    if (actor === this.loaded) return;
    this.loaded = actor;
    this.avatar = null;
    if (!browser || !actor) return;

    // the demo wall is a fixture with no real profile; give it a stable face
    // from the same source the fixture bricks use
    if (actor === "demo") {
      this.avatar = "https://picsum.photos/seed/masondemo/96/96";
      return;
    }

    void fetch(`${APPVIEW}/xrpc/app.bsky.actor.getProfile?actor=${encodeURIComponent(actor)}`)
      .then((res) => (res.ok ? res.json() : null))
      .then((data: { avatar?: string } | null) => {
        // ignore a response that arrived after the handle changed under us
        if (this.loaded !== actor || !data) return;
        this.avatar = data.avatar ?? null;
      })
      .catch(() => {
        // no face; the button shows an initial instead
      });
  }
}

export const profile = new ProfileState();
