// Everything the settings screen decides: the history entry that holds it open,
// and the fact that opening it shuts whichever overlay was up.
//
// `Settings.svelte` renders it and no lane in this repo typechecks or runs a
// component body, which is exactly why none of these decisions live in it.
import { beforeEach, describe, expect, it, vi } from "vitest";
import { pushState, replaceState } from "$app/navigation";
import { SettingsState, settings } from "./settings.svelte";

vi.mock("$app/navigation", () => ({ pushState: vi.fn(), replaceState: vi.fn() }));

// `page.state` is the screen's open/shut signal and the real one needs a live
// router. The getter defers the reference to call time, because a vi.mock
// factory is hoisted above this module's own initialization.
const pageState: App.PageState = {};
vi.mock("$app/state", () => ({
  page: {
    get state() {
      return pageState;
    },
  },
}));

const back = vi.fn();

beforeEach(() => {
  back.mockReset();
  vi.mocked(pushState).mockReset();
  vi.mocked(replaceState).mockReset();
  // `history` is a browser global with nothing behind it in node; the own-entry
  // branch of closeSettings is the only thing that reaches for it
  vi.stubGlobal("history", { back });
  delete pageState.settings;
  delete pageState.picker;
  delete pageState.brick;
});

describe("settings is history, not a route", () => {
  it("opens on its own key alone, which is what shuts the other overlays", () => {
    pageState.brick = "a"; // the reader is up over a wall
    pageState.picker = "feeds"; // and, impossibly, the picker too
    const screen = new SettingsState();

    screen.openSettings();

    const pushed = vi.mocked(pushState).mock.calls[0]?.[1];
    expect(pushed).toEqual({ settings: true });
    // the load-bearing half: a push REPLACES page state rather than merging into
    // it, so carrying only this key is the whole of the mutual-exclusion rule.
    // The other two halves have the same shape and their own tests pin them.
    expect(pushed).not.toHaveProperty("brick");
    expect(pushed).not.toHaveProperty("picker");
    expect(pushState).toHaveBeenCalledExactlyOnceWith("", { settings: true });
  });

  it("is up only while page.state says so", () => {
    const screen = new SettingsState();
    expect(screen.isOpen).toBe(false);

    screen.openSettings();
    pageState.settings = true; // the router's own update, once the push lands
    expect(screen.isOpen).toBe(true);

    delete pageState.settings; // the back gesture, which never calls closeSettings
    expect(screen.isOpen).toBe(false);
  });

  it("does not stack a second entry when it is already open", () => {
    const screen = new SettingsState();
    screen.openSettings();
    pageState.settings = true;
    // a second tap on the cog while the screen is up; two entries would need two
    // back gestures to leave
    screen.openSettings();
    expect(pushState).toHaveBeenCalledTimes(1);
  });

  it("pops the entry it pushed", () => {
    const screen = new SettingsState();
    screen.openSettings();
    screen.closeSettings();
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).not.toHaveBeenCalled();
  });

  it("replaces the state when the entry is not its own", () => {
    // nothing was pushed (a reload landed on the entry, say), so going back would
    // leave mason rather than close the screen
    new SettingsState().closeSettings();
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
    expect(back).not.toHaveBeenCalled();
  });

  it("spends its entry once, whichever way it went", () => {
    const screen = new SettingsState();
    screen.openSettings();
    screen.closeSettings();
    screen.closeSettings(); // a second escape, or a scrim click racing the back
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
  });

  it("exports a singleton, which is what the cog and the screen share", () => {
    expect(settings).toBeInstanceOf(SettingsState);
    expect(settings.isOpen).toBe(false);
  });
});
