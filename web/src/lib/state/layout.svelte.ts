import { browser } from "$app/environment";

/** How the wall lays its bricks. Bento gives feature bricks a wider footprint
 *  on a dense grid; masonry packs even-width bricks into the shortest column;
 *  glaze is the image wall — a dense bento of nothing but pictures at their own
 *  perspective, the gaps grouted with muted filler. One reader preference,
 *  remembered like the client picker. Picking glaze also switches the feed to
 *  the images-only algorithm (see FeedGrid / +page). */
export const LAYOUTS = [
  { id: "bento", label: "Bento", icon: "🍱" },
  { id: "masonry", label: "Masonry", icon: "🧱" },
  { id: "glaze", label: "Glaze", icon: "🏺" },
] as const;

export type LayoutId = (typeof LAYOUTS)[number]["id"];

const STORAGE_KEY = "mason:layout";
const DEFAULT: LayoutId = "bento";

function stored(): LayoutId {
  if (!browser) return DEFAULT;
  const saved = localStorage.getItem(STORAGE_KEY);
  return LAYOUTS.some((l) => l.id === saved) ? (saved as LayoutId) : DEFAULT;
}

class LayoutState {
  id = $state<LayoutId>(stored());

  set(id: LayoutId) {
    this.id = id;
    if (browser) localStorage.setItem(STORAGE_KEY, id);
  }
}

export const layout = new LayoutState();
