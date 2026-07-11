import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			// pure static SPA in both modes; local mode's feed engine is the
			// wasm service worker, server mode calls mortar over CORS
			adapter: adapter({ fallback: 'index.html' }),
			serviceWorker: {
				// registered manually in +layout.svelte, local mode only
				register: false
			}
		})
	],
	optimizeDeps: {
		// wasm-pack output must reach the browser untouched
		exclude: ['$lib/mortar-wasm/pkg']
	},
	define: {
		// Build-mode switch with a hard default: unset → '' → local mode
		// (wasm service worker). A real env var — no .env file to forget in
		// deploy zips; `just dev-server` sets it for server mode.
		'import.meta.env.PUBLIC_MASON_SERVER_URL': JSON.stringify(
			process.env.PUBLIC_MASON_SERVER_URL ?? ''
		)
	}
});
