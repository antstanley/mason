// The shared reveal set is the whole of the "uncover once, stays uncovered"
// behaviour: `Sensitive` covers a brick whenever its id is absent from here.
// That component is a .svelte file and no lane in the repo renders one, so
// these pin the set itself, which is the half that can be tested.
import { afterEach, describe, expect, it } from "vitest";
import { SvelteSet } from "svelte/reactivity";
import { revealed } from "./sensitive.svelte";

// a module singleton, so every case hands it back empty. The empty-start case
// runs first, before anything has added to it, which is what makes it honest.
afterEach(() => revealed.clear());

describe("revealed", () => {
  it("starts empty, so a !warn brick arrives covered", () => {
    expect(revealed.size).toBe(0);
    expect(revealed.has("fixture-post-0")).toBe(false);
  });

  it("is a reactive set, so a reveal re-renders the brick that shows it", () => {
    // a plain Set would satisfy every other case here and quietly break the
    // reveal: nothing would re-run when an id landed in it
    expect(revealed).toBeInstanceOf(SvelteSet);
  });

  it("takes the same id twice without growing", () => {
    revealed.add("fixture-post-0");
    revealed.add("fixture-post-0");
    expect(revealed.size).toBe(1);
    expect(revealed.has("fixture-post-0")).toBe(true);
  });

  it("keys the choice per brick, so one reveal does not uncover the wall", () => {
    revealed.add("fixture-post-0");
    expect(revealed.has("fixture-post-0")).toBe(true);
    expect(revealed.has("fixture-post-3")).toBe(false);
  });
});
