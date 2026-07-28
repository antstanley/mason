import { createHash } from "node:crypto";
import { defineConfig, devices } from "@playwright/test";

// The specs in tests/ run against the real static build. They are the only lane
// in this repo that renders a component at all, because tsc cannot parse a
// .svelte file and both vitest suites are .ts: the service worker smoke, the
// brick reader, the feed picker, the refresh control and the repeated-tag
// regression all live or die here.

// The preview port is derived from this checkout's own path rather than fixed.
// `reuseExistingServer` attaches to whatever already holds the port and has no
// way to ask which tree built it, so on a machine running two checkouts (a jj
// workspace beside the main one, say) a fixed port means one run can silently
// drive the other's build and report green. That is not hypothetical: it
// happened here, and a verifier caught it mid-review rather than the gate.
// Deriving the port means a reused server can only ever be this checkout's own,
// and `--strictPort` turns anything else into a loud failure instead of a quiet
// wrong answer. The range stays well below the ephemeral ports the OS hands out.
const PORT_BASE = 4173;
const PORT_SPAN = 800;
const digest = createHash("sha256").update(import.meta.dirname).digest("hex");
const port = PORT_BASE + (Number.parseInt(digest.slice(0, 6), 16) % PORT_SPAN);
const origin = `http://localhost:${port}`;

export default defineConfig({
	testDir: "tests",
	// Refuses to run when web/build/ is older than what it was built from. The
	// preview server is started first and torn straight back down, so a stale
	// tree costs a couple of seconds and not a suite. See playwright.setup.ts for
	// what this looks like WITHOUT the guard, which is a green run against code
	// that has already been deleted.
	globalSetup: "./playwright.setup.ts",
	timeout: 60_000,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	use: {
		baseURL: origin,
		// a retried pass must still leave evidence of the first failure
		trace: "on-first-retry",
	},
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
	webServer: {
		// serves web/build/; run `just build` (or `just test-e2e`) first
		command: `pnpm preview --port ${port} --strictPort`,
		url: origin,
		reuseExistingServer: !process.env.CI,
		timeout: 30_000,
	},
});
