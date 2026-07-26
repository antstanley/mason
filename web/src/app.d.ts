// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    /** Shallow-routing state, one optional member per in-place surface. It is
     *  never serialized and never survives a reload, so every member has to be
     *  optional and a reload lands on the plain wall. */
    interface PageState {
      /** The id of the brick being read in place; unset means no reader. The
       *  brick itself lives in the `reader` rune, since page state travels
       *  through history and a `Brick` does not. */
      brick?: string;
    }
    // interface Platform {}
  }
}

export {};
