import { defineConfig, devices } from "@playwright/test";

// One smoke test against the real static build: the wasm service worker must
// intercept /api/feed and lay the demo wall (offline fixtures, no network).
export default defineConfig({
	testDir: "tests",
	timeout: 60_000,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	use: {
		baseURL: "http://localhost:4173",
		// a retried pass must still leave evidence of the first failure
		trace: "on-first-retry",
	},
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
	webServer: {
		// serves web/build/; run `just build` (or `just test-e2e`) first
		command: "pnpm preview --port 4173 --strictPort",
		url: "http://localhost:4173",
		reuseExistingServer: !process.env.CI,
		timeout: 30_000,
	},
});
