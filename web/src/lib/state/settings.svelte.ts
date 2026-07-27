// The settings screen: the third thing mason opens over a wall, after the brick
// reader and the feed picker.
//
// It holds the preferences a reader sets once and then forgets, which is the
// whole reason it exists as its own surface rather than as another control on
// the bar. The bar is for what changes while you read: which layout, lay the
// wall again, which wall. Which client a post opens in is not that.
//
// Like both of its siblings it is a screen, not a route. `?actor=` and `?feed=`
// are the whole routing surface (07-web-client.md), and settings is not a wall,
// so it lives in history state: the address bar keeps showing the wall behind
// it, and the back gesture closes it rather than leaving mason.
//
// This module is the same shape as the picker's half of `feeds.svelte.ts`, on
// purpose and not by accident: nothing in this repo typechecks or runs a
// component body, so a history rule kept in `Settings.svelte` would be a rule
// nothing can check. Three overlays now push and pop the same way, which is one
// more than makes duplication comfortable; a shared helper is the obvious next
// move, and it is deliberately not made here because it would rewrite two
// modules that already ship with their own tests and specs.
import { pushState, replaceState } from "$app/navigation";
import { page } from "$app/state";

export class SettingsState {
  /** Whether the entry the screen is sitting on is ours to pop. A reload lands
   *  on somebody else's entry, and going back from that leaves mason. */
  #pushed = false;

  /** Whether the screen is up. `page.state` decides, and nothing else does:
   *  `+layout.svelte` makes the wall inert behind whichever overlay is open, and
   *  a wrapper made inert on a wider condition than the screen renders on is a
   *  page frozen under nothing. */
  get isOpen(): boolean {
    return page.state.settings === true;
  }

  /** Open settings over whatever is behind it.
   *
   *  The pushed state carries this key and NOTHING else, which is the whole of
   *  the mutual-exclusion rule the three overlays share: a push replaces
   *  `App.PageState` rather than merging into it, so this drops `brick` and
   *  `picker`, and the reader and the picker shut. Neither of them has to know
   *  settings exists. */
  openSettings() {
    // already up: a second push would leave a second entry to walk back through
    if (this.isOpen) return;
    pushState("", { settings: true });
    this.#pushed = true;
  }

  /** Shut settings. */
  closeSettings() {
    const ours = this.#pushed;
    // spent either way: whichever branch runs, that entry is dealt with once
    this.#pushed = false;
    if (ours) {
      // pop our own entry, so the screen leaves no rubble in the history stack.
      // The router clears `page.state` as the popstate lands, which is what
      // shuts the screen; nothing here has to.
      history.back();
      return;
    }
    // no entry of ours to pop, so drop the key from the current one instead
    replaceState("", {});
  }
}

export const settings = new SettingsState();
