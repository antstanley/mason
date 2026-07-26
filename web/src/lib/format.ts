// Display strings for the two brick fields a card never had room for: how long
// a video runs, and when a brick was made.
//
// They live in a plain `.ts` module rather than in the component that renders
// them because this file is typechecked and unit tested and a `.svelte` body is
// neither, and because two components need the runtime: the video card's badge
// and the brick reader's, which must read the same for the same brick.

/** A video's runtime, in hours and minutes.
 *
 *  A stream runs long enough that seconds are noise, but not every archived
 *  video is a long one: clips of a few seconds exist, and rounding those to
 *  "0m" makes the card look broken, so anything under a minute keeps its
 *  seconds. */
export function runtimeLabel(ms: number): string {
  // a duration that is missing, zero or nonsense gets no label at all rather
  // than a badge reading "0s": callers render the badge only when this comes
  // back with something in it. The wire's durationMs is nullable and mortar
  // fills it from upstream metadata, which is where a NaN would come from.
  if (!Number.isFinite(ms) || ms <= 0) return "";
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.round(seconds / 60);
  const hours = Math.floor(minutes / 60);
  return hours > 0 ? `${hours}h ${minutes % 60}m` : `${minutes}m`;
}

/** When a brick was made, in the viewer's own locale and time zone.
 *
 *  `locale` exists so a test can pin the wording; the app never passes one,
 *  which is what leaves the formatting to the browser the reader is holding. */
export function dateLabel(iso: string, locale?: Intl.LocalesArgument): string {
  const at = new Date(iso);
  // timestamps are opaque strings on the wire (mortar passes upstream's through
  // untouched), so an unparseable one is reachable from here. It yields NaN,
  // and a brick stamped "Invalid Date" is worse than a brick with no stamp:
  // callers render the line only when this comes back with something in it.
  if (Number.isNaN(at.getTime())) return "";
  return at.toLocaleString(locale, { dateStyle: "medium", timeStyle: "short" });
}
