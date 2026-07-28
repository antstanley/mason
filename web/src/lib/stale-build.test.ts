// The guard that refuses to let an e2e run answer from a stale build
// (`web/playwright.setup.ts`).
//
// It is tested because of what it does when it is wrong. A guard that fires
// when it should not is loud and gets fixed in a minute; this one fails the
// other way. If a directory moves and the walk quietly finds nothing, it
// reports a fresh build forever and hands the whole problem back: `vite preview`
// serves `web/build/` without compiling, so the suite answers about code that
// has been deleted. Measured on one tree, by renaming the reader's "next brick"
// control: 18 passed without a rebuild, and 2 failed with one.
//
// Driven against temporary directories rather than the repo, so a case here
// cannot depend on whether somebody happens to have run `just build` recently.
import { mkdtempSync, mkdirSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { staleBuild } from "../../playwright.setup";

let root: string;
const made: string[] = [];

/** A throwaway tree with a `src` and a `build`, and nothing else. */
function tree() {
  root = mkdtempSync(join(tmpdir(), "mason-stale-"));
  made.push(root);
  mkdirSync(join(root, "src"), { recursive: true });
  mkdirSync(join(root, "build"), { recursive: true });
  return { src: join(root, "src"), build: join(root, "build") };
}

/** Write a file and pin its mtime, in seconds since the epoch. Times are set
 *  rather than taken from the clock: a test that wrote two files and hoped they
 *  landed in different milliseconds would be a coin toss on a fast disk. */
function file(dir: string, name: string, at: number) {
  const path = join(dir, name);
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, name);
  utimesSync(path, at, at);
  return path;
}

afterEach(() => {
  for (const dir of made.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("staleBuild", () => {
  it("says nothing when the build is newer than everything it was made from", () => {
    const { src, build } = tree();
    file(src, "app.svelte", 1000);
    file(build, "index.html", 2000);

    expect(staleBuild([src], build)).toBeNull();
  });

  it("complains, and names the file, when a source is newer than the build", () => {
    const { src, build } = tree();
    file(build, "index.html", 1000);
    file(src, "app.svelte", 2000);

    const said = staleBuild([src], build);
    expect(said).toContain("older than 1 of the files");
    expect(said).toContain("app.svelte");
    // and it says what to do about it, which is the whole reason it exists
    expect(said).toContain("just build");
    expect(said).toContain("just test-e2e");
  });

  it("complains when there is no build at all", () => {
    const { src } = tree();
    file(src, "app.svelte", 1000);

    const said = staleBuild([src], join(root, "nothing-here"));
    expect(said).toContain("no build");
    expect(said).toContain("just build");
  });

  it("finds a stale file nested well below the input root", () => {
    // the walk has to recurse: components live several directories down, and a
    // guard that only looked at the top level would pass on almost every real
    // edit anybody makes
    const { src, build } = tree();
    file(build, "index.html", 1000);
    file(src, "lib/components/cards/PostCard.svelte", 2000);

    expect(staleBuild([src], build)).toContain("PostCard.svelte");
  });

  it("watches every input it is given, not just the first", () => {
    // `server/crates` is the one this is really about: a Rust change reaches the
    // browser through `just wasm`, and it is the staleness nobody suspects
    // because nothing under web/ was touched.
    const { src, build } = tree();
    const crates = join(root, "crates");
    mkdirSync(crates, { recursive: true });
    file(build, "index.html", 1000);
    file(src, "app.svelte", 500);
    file(crates, "mortar-core/src/feed.rs", 2000);

    expect(staleBuild([src, crates], build)).toContain("feed.rs");
  });

  it("ignores a missing input rather than throwing", () => {
    // `web/static` need not exist, and a guard that crashed on that would be a
    // worse failure than the one it is preventing
    const { src, build } = tree();
    file(build, "index.html", 2000);
    file(src, "app.svelte", 1000);

    expect(staleBuild([src, join(root, "not-there")], build)).toBeNull();
  });

  it("does not walk into node_modules or target", () => {
    // both are enormous and neither is an input. Walking them would cost more
    // than the build this is guarding, and a stray mtime in either would fire
    // the guard for no reason anybody could act on.
    const { src, build } = tree();
    file(build, "index.html", 1000);
    file(src, "node_modules/left-pad/index.js", 9000);
    file(src, "target/debug/whatever", 9000);

    expect(staleBuild([src], build)).toBeNull();
  });

  it("ignores a generated directory that sits inside an input", () => {
    // web/src/lib/mortar-wasm/pkg is written by `just wasm`, which rewrites it
    // whole on every run whether or not any Rust changed, and `just check`
    // depends on wasm. Without this the local gate would leave the tree looking
    // stale to every playwright run after it. Its real source is server/crates,
    // which is watched separately, so a Rust change still fires.
    const { src, build } = tree();
    const pkg = join(src, "lib/mortar-wasm/pkg");
    file(build, "index.html", 1000);
    file(pkg, "mortar_wasm.js", 9000);

    expect(staleBuild([src], build, [pkg])).toBeNull();
    // and without the exclusion it is exactly the false positive described
    expect(staleBuild([src], build)).toContain("mortar_wasm.js");
  });

  it("names the newest first and stops before the list becomes a wall", () => {
    // a branch switch restamps the whole tree, and a thousand paths is a list
    // nobody reads
    const { src, build } = tree();
    file(build, "index.html", 1000);
    for (let i = 0; i < 12; i++) file(src, `file-${i}.ts`, 2000 + i);

    const said = staleBuild([src], build) ?? "";
    expect(said).toContain("older than 12 of the files");
    expect(said).toContain("file-11.ts"); // the newest, listed
    expect(said).toContain("and 4 more"); // 12 stale, 8 shown
    expect(said).not.toContain("file-0.ts"); // the oldest, past the cut
  });
});
