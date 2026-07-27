// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    /** Shallow-routing state, one optional member per in-place surface. It is
     *  never serialized and never survives a reload, so every member has to be
     *  optional and a reload lands on the plain wall.
     *
     *  One member is set at a time, and that is the whole of the rule about the
     *  overlays: a push REPLACES this object rather than merging into it, so
     *  `pushState('', { brick })` in state/reader.svelte.ts,
     *  `pushState('', { picker: 'feeds' })` in state/feeds.svelte.ts and
     *  `pushState('', { settings: true })` in state/settings.svelte.ts each
     *  clear the others' keys by carrying only their own. The picker is a
     *  landing surface, the reader is a wall surface and settings is neither,
     *  and none of them has anything to say over the top of another. */
    interface PageState {
      /** The id of the brick being read in place; unset means no reader. The
       *  brick itself lives in the `reader` rune, since page state travels
       *  through history and a `Brick` does not. */
      brick?: string;
      /** The feed picker, mason's second front door, open over whatever is
       *  behind it. A literal rather than a boolean because the picker is a
       *  screen and not a route: this key names WHICH screen, so a second one
       *  could be added without every reader of this member changing shape. */
      picker?: "feeds";
      /** The settings screen, open over whatever is behind it. A boolean rather
       *  than a literal because there is one of it: settings is not a screen
       *  among several the way the picker's key allows for, it is the place the
       *  preferences a reader sets once and forgets live. */
      settings?: true;
    }
    // interface Platform {}
  }
}

export {};
