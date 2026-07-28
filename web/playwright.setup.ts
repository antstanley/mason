// The guard that stands between a source edit and a green run against the build
// before it.
//
// `vite preview` SERVES `web/build/`. It does not compile, and playwright starts
// it for you, so `pnpm exec playwright test` after an edit to `web/src` drives
// the previous build and reports on code that no longer exists. It is not a
// flake and it does not look like one: the run is fast, green and completely
// wrong.
//
// Measured, on one tree, three ways. Rename the reader's "next brick" control,
// which two cases in reader.test.ts assert on by name:
//
//     no rebuild, no guard   18 passed      <- a lie
//     no rebuild, guard      refuses        <- this file
//     rebuild                2 failed       <- the truth
//
// The first line is the whole reason this exists. It is not that the suite went
// quiet; it is that it answered confidently about a tree nobody has.
//
// `just test-e2e` never had the problem, because it depends on `build`. The trap
// is the shortcut everybody reaches for to run one spec, and the config's own
// "run `just build` first" comment is only ever read by somebody who already
// suspects. So this is a check rather than a sentence: it compares what the
// build was made from against what is in the build, and refuses to let the run
// start when the answer would be about the wrong tree.
//
// It stays quiet during the loop that matters. `web/tests/` is not a build
// input, so editing a spec and rerunning it, over and over, never trips this.
// Only a change to something the browser actually serves does.
import { existsSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const WEB = import.meta.dirname;
const REPO = join(WEB, "..");
const BUILD = join(WEB, "build");

/** Everything the static build is made from.
 *
 *  `server/crates` is on the list and is not a mistake: the wall is laid by
 *  mortar compiled to wasm, `just wasm` writes that into `web/src/lib/
 *  mortar-wasm/pkg/`, and vite bundles it. A Rust change with no rebuild behind
 *  it is the same stale run wearing a different hat, and it is the one people
 *  are least likely to suspect, because nothing under `web/` was touched. */
const INPUTS = [
	join(WEB, "src"),
	join(WEB, "static"),
	join(WEB, "vite.config.ts"),
	join(WEB, "svelte.config.js"),
	join(WEB, "package.json"),
	join(REPO, "server", "crates"),
	// a dependency bump changes the wasm without changing a line of rust
	join(REPO, "server", "Cargo.toml"),
	join(REPO, "server", "Cargo.lock"),
];

/** Paths under an input that are OUTPUT and must not be read as changes.
 *
 *  `mortar-wasm/pkg/` sits inside `web/src`, but it is written by `just wasm`,
 *  which rewrites every file in it on every invocation whether or not any Rust
 *  changed. `just check` depends on `wasm`, so without this the local gate would
 *  leave the tree looking stale to every playwright run after it, forever, for
 *  no reason anybody could act on. The wasm's real source is `server/crates`,
 *  which is on the list above, so a Rust change still fires this and a rebuild
 *  that changed nothing does not. */
const GENERATED = [join(WEB, "src", "lib", "mortar-wasm", "pkg")];

/** Directories that are output or machinery, never input. `target` is cargo's
 *  and is enormous; walking it would cost more than the build it is guarding. */
const SKIP = new Set(["node_modules", "target", ".svelte-kit", ".git", ".jj"]);

interface Newest {
	path: string;
	at: number;
}

/** The most recently touched file under a path, or null if there is nothing
 *  there. Follows directories and ignores the machinery in SKIP. */
function newest(path: string): Newest | null {
	if (!existsSync(path)) return null;
	const stat = statSync(path);
	if (!stat.isDirectory()) return { path, at: stat.mtimeMs };

	let found: Newest | null = null;
	for (const entry of readdirSync(path, { withFileTypes: true })) {
		if (SKIP.has(entry.name)) continue;
		const child = newest(join(path, entry.name));
		if (child && (!found || child.at > found.at)) found = child;
	}
	return found;
}

/** Every input newer than the build, newest first, for the message. Reported
 *  rather than counted: "something changed" sends somebody looking, and the
 *  whole point of this is to not waste the cycle it just saved. */
function newerThan(inputs: string[], at: number, generated: string[]): string[] {
	const stale: Newest[] = [];
	const walk = (path: string) => {
		if (!existsSync(path) || generated.includes(path)) return;
		const stat = statSync(path);
		if (!stat.isDirectory()) {
			if (stat.mtimeMs > at) stale.push({ path, at: stat.mtimeMs });
			return;
		}
		for (const entry of readdirSync(path, { withFileTypes: true })) {
			if (!SKIP.has(entry.name)) walk(join(path, entry.name));
		}
	};
	for (const input of inputs) walk(input);
	return stale.sort((a, b) => b.at - a.at).map((f) => relative(REPO, f.path));
}

/** How many stale paths the message names before it stops. After a branch
 *  switch this can be the whole tree, and a thousand paths is a wall of text
 *  nobody reads. */
const SHOWN = 8;

const RUN_INSTEAD = [
	"    just build      then run playwright again",
	"    just test-e2e   builds and runs in one go",
];

/** The complaint, or null when the build can answer for the tree.
 *
 *  Separated from the hook and given its own paths so it can be tested against
 *  temporary directories: a guard nothing exercises is a guard that stops
 *  guarding the first time somebody moves a directory, and this one would fail
 *  open, silently, into exactly the behaviour it exists to prevent. */
export function staleBuild(inputs: string[], build: string, generated: string[] = []): string | null {
	const built = newest(build);
	if (!built) {
		return [
			"",
			"There is no build for these specs to run against.",
			"",
			"The specs in web/tests/ drive the real static site, and `vite preview`",
			"only serves it. Nothing here compiles.",
			"",
			...RUN_INSTEAD,
			"",
		].join("\n");
	}

	const stale = newerThan(inputs, built.at, generated);
	if (stale.length === 0) return null;

	const shown = stale.slice(0, SHOWN);
	const rest = stale.length - shown.length;
	return [
		"",
		`This build is older than ${stale.length} of the files it was made from.`,
		"",
		"`vite preview` serves web/build/ and does not compile, so this run would",
		"have driven the PREVIOUS build and told you about code you no longer have.",
		"That failure is green, fast and silent, which is why it is a hard stop.",
		"",
		...shown.map((path) => `    ${path}`),
		...(rest > 0 ? [`    ... and ${rest} more`] : []),
		"",
		...RUN_INSTEAD,
		"",
		"(editing a spec in web/tests/ never trips this: specs are not built)",
		"",
	].join("\n");
}

export default function guardFreshBuild(): void {
	const complaint = staleBuild(INPUTS, BUILD, GENERATED);
	if (complaint) throw new Error(complaint);
}
