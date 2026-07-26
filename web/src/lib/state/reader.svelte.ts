import { pushState, replaceState } from "$app/navigation";
import { page } from "$app/state";
import { feed } from "./feed.svelte";
import type { Brick } from "$lib/types";

/** What the reader needs from the element a brick was opened from: somewhere to
 *  put focus back when it closes. Structural rather than `HTMLElement`, so this
 *  module names no DOM global and vitest can run it for real in node. */
interface Focusable {
  focus: () => void;
}

/** The half of a `MouseEvent` the activation rule reads. Structural for the same
 *  reason `Focusable` is: a real click event satisfies it, and a test can hand
 *  `activate` a plain object without a DOM behind it. */
interface Activation {
  metaKey: boolean;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  button: number;
  currentTarget: EventTarget | null;
  preventDefault: () => void;
}

/** Narrow a click's target to something focusable without naming `HTMLElement`.
 *  An `instanceof` against that global would throw where these tests run, and
 *  the only thing the reader ever asks an opener for is `focus()`. The cast is
 *  justified by the `typeof` guarding it: nothing is handed back unless `focus`
 *  really is callable. */
function focusable(target: EventTarget | null): Focusable | null {
  const candidate = target as Focusable | null;
  return typeof candidate?.focus === "function" ? candidate : null;
}

/** The brick reader: one brick read in place, over the wall it came from.
 *
 *  `page.state.brick` is the single source of truth for whether the reader is up
 *  (`App.PageState` in `app.d.ts`), which is what makes the back gesture close
 *  it. This rune holds the `Brick` itself, because page state carries an id and
 *  mason has no way to fetch one brick by id: the wall is the only place it
 *  exists. Every decision the reader makes lives here rather than in the dialog,
 *  since a `.svelte.ts` rune module is typechecked and tested and a `.svelte`
 *  component body is neither.
 *
 *  Exported for the unit tests, which build throwaway instances; the app only
 *  ever uses the `reader` singleton below. */
export class ReaderState {
  /** The brick being read. Held, never looked up: see the note above. */
  brick = $state<Brick | null>(null);

  /** The card that opened the reader, so focus can go back to it. */
  #opener: Focusable | null = null;

  /** Whether the history entry the reader is sitting on is ours to pop. */
  #pushed = false;

  /** Whether the reader is up. `page.state` decides that and `brick` does not:
   *  the back gesture clears the state without ever calling `close`, and the
   *  brick is deliberately still held afterwards so a dialog on its way out
   *  never reads a null one mid-teardown. */
  get isOpen(): boolean {
    return page.state.brick !== undefined;
  }

  /** Where the open brick sits on the wall, located by id on every read rather
   *  than stored: a reordered or replaced `feed.items` then yields -1, which
   *  reads as "no longer on the wall", instead of quietly pointing at whichever
   *  brick took the old slot. It is a `findIndex` over a few hundred items, once
   *  per step, which is cheaper than threading an index through the wall. */
  get index(): number {
    const open = this.brick;
    if (!open) return -1;
    return feed.items.findIndex((b) => b.id === open.id);
  }

  get canPrev(): boolean {
    return this.index > 0;
  }

  get canNext(): boolean {
    const at = this.index;
    // -1 fails the first half, so a brick that left the wall steps neither way
    return at >= 0 && at < feed.items.length - 1;
  }

  /** Open `brick` in place. `opener` is the element the click came from, which
   *  is where focus returns on close. */
  open(brick: Brick, opener: Focusable | null = null) {
    // a click is engagement, so commit the arrangement before anything else:
    // while the wall is warming it reorders between preview polls, and the
    // reader locates its brick by id in exactly that list. Fire and forget,
    // because freeze is async and the reader opens on this click, not a fetch
    // later; freeze handles its own failures and settles a second caller as a
    // no-op, so there is no rejection to chase here.
    void feed.freeze();
    // the push comes before the state changes, because it is the one step here
    // that can throw (a router that is not up yet), and when it does the reader
    // must stay shut rather than hold a brick no dialog will ever render
    pushState("", { brick: brick.id });
    this.#pushed = true;
    this.brick = brick;
    this.#opener = opener;
  }

  /** The one modifier-key rule, shared by all three activation points (the
   *  card-wide anchor, the video card's watch link, the glaze card's images).
   *  Returns whether the reader took the click. */
  activate(event: Activation, brick: Brick): boolean {
    // a modified click is the reader asking the browser for the source itself:
    // a new tab, a new window, a download, a saved link. Never intercept one,
    // and never intercept anything but the primary button, because a middle
    // click is "open in a new tab" spelled another way. Keeping the anchor and
    // declining these clicks is what preserves the browser's own affordances.
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0) {
      return false;
    }
    event.preventDefault();
    this.open(brick, focusable(event.currentTarget));
    return true;
  }

  /** Shut the reader. */
  close() {
    const ours = this.#pushed;
    // spent either way: whichever branch runs, that entry is dealt with once
    this.#pushed = false;
    if (ours) {
      // pop our own entry, so the reader leaves no rubble behind it in the
      // history stack and the wall's entry becomes current again. The router
      // clears `page.state` as the popstate lands, which is what shuts the
      // dialog; nothing here has to.
      history.back();
      return;
    }
    // no entry of ours to pop, so drop the brick from the current one instead.
    // Going back from here would leave the wall altogether, which is the
    // opposite of closing a reader.
    replaceState("", {});
  }

  next() {
    this.#step(1);
  }

  prev() {
    this.#step(-1);
  }

  /** Hand focus back to the card that opened the reader. The dialog calls this
   *  as it tears down, not `close`: `close` only asks history to pop, and the
   *  dialog is still mounted and still holding focus when it returns. */
  returnFocus() {
    this.#opener?.focus();
  }

  /** Step one brick along the laid wall. */
  #step(delta: 1 | -1) {
    const at = this.index;
    if (at < 0) return; // the open brick left the wall: nowhere to step from
    const to = feed.items[at + delta];
    // the ends of the laid wall are the ends of the reader: stepping never
    // paginates, because a page fetched from in here would grow a wall this
    // reader cannot see. `noUncheckedIndexedAccess` makes both ends one check,
    // since index -1 and index length are both `undefined`.
    if (!to) return;
    this.brick = to;
    // replace rather than push: a step is not a new history entry, so one back
    // gesture still closes the reader instead of walking it back brick by
    // brick, and the entry stays ours, so `#pushed` is left alone
    replaceState("", { brick: to.id });
  }
}

export const reader = new ReaderState();
