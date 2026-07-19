import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

// ride the app's own vite config so .svelte.ts rune modules compile in tests
// exactly as they do in the build
export default mergeConfig(
	viteConfig,
	defineConfig({
		test: {
			include: ["src/**/*.test.ts"],
			environment: "node",
		},
	}),
);
