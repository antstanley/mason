// The two display strings the reader added, both of which have a shape a
// component cannot assert about itself: no lane in this repo typechecks or
// renders a `.svelte` body, so the formatting lives here and is pinned here.
import { describe, expect, it } from "vitest";
import { dateLabel, runtimeLabel } from "./format";

describe("runtimeLabel", () => {
  it("keeps the seconds on a clip too short to round", () => {
    // a 42 second clip rounded to minutes reads "1m", and rounded down "0m",
    // which is the case that made this keep two units at all
    expect(runtimeLabel(42_000)).toBe("42s");
  });

  it("reads in whole minutes under an hour", () => {
    expect(runtimeLabel(600_000)).toBe("10m");
  });

  it("splits hours from minutes above one", () => {
    // the demo wall's archived stream: 4_920_000ms is 82 minutes
    expect(runtimeLabel(4_920_000)).toBe("1h 22m");
  });

  it.each([
    ["nothing at all", 0],
    ["a negative duration", -1000],
    ["a duration that is not a number", Number.NaN],
    ["an infinite duration", Number.POSITIVE_INFINITY],
  ])("has no label for %s", (_name, ms) => {
    // the badge is rendered on a non-empty label, so an unusable duration
    // shows nothing rather than "0s" or "NaNm"
    expect(runtimeLabel(ms)).toBe("");
  });
});

describe("dateLabel", () => {
  it("reads as a date and a time of day", () => {
    // the clock reading depends on the machine's time zone and the day can tip
    // either side of it, so this pins the SHAPE rather than the hour: a medium
    // date, then a short time. Asserting the exact string would be asserting
    // where the test happens to be running.
    expect(dateLabel("2026-01-15T12:00:00Z", "en-GB")).toMatch(/^1[456] Jan 2026, \d{2}:\d{2}$/);
  });

  it("follows the locale it is given", () => {
    expect(dateLabel("2026-01-15T12:00:00Z", "en-US")).toMatch(
      /^Jan 1[456], 2026, \d{1,2}:\d{2}\s?[AP]M$/,
    );
  });

  it.each([
    ["an empty string", ""],
    ["prose", "tomorrow"],
    ["a truncated timestamp", "2026-13"],
  ])("has no label for %s", (_name, iso) => {
    // the line is rendered on a non-empty label, so a brick whose upstream
    // timestamp is junk shows no date rather than "Invalid Date"
    expect(dateLabel(iso, "en-GB")).toBe("");
  });
});
