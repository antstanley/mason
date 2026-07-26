import { SvelteSet } from "svelte/reactivity";

/** Brick ids whose `!warn` media the reader has uncovered. The choice follows
 *  the brick rather than the element that rendered it, so a brick uncovered on
 *  the wall is still uncovered wherever it is rendered next: after a layout
 *  switch re-places it, or in a second view of the same brick. Finding it
 *  covered again one click later reads as a bug.
 *
 *  Session-scoped on purpose. It is a rune, never storage, so a reload forgets
 *  every reveal and no lingering "show everything" switch can accumulate. */
export const revealed = new SvelteSet<string>();
