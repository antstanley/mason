<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import favicon from '$lib/assets/favicon.svg';
	import { localMode } from '$lib/api';
	import { handle } from '$lib/state/handle.svelte';

	let { children } = $props();

	// local mode: the wasm service worker IS the feed server.
	// Always type module: the wasm-bindgen glue contains `import.meta`,
	// which a classic script rejects at parse time. Module SWs are
	// everywhere in 2026 (Chrome 91+, Safari 15+, Firefox 147+).
	$effect(() => {
		if (!browser || !localMode || !('serviceWorker' in navigator)) return;
		void navigator.serviceWorker.register('/service-worker.js', { type: 'module' });
	});
</script>

<svelte:head>
	<title>mason — an atproto discovery wall</title>
	<link rel="icon" href={favicon} />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:wght@400;700;800&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="mx-auto min-h-screen max-w-[1800px] px-4 sm:px-6">
	{#if handle.current}
		<header class="flex items-center justify-between py-4">
			<p class="font-display text-2xl font-black tracking-tight">
				mason <span class="text-pop-pink">🧱</span>
			</p>
			<div class="flex items-center gap-3 text-sm">
				<span class="font-semibold opacity-70">@{handle.current}</span>
				<button
					type="button"
					onclick={() => handle.clear()}
					class="cursor-pointer rounded-full border-2 border-ink/15 px-3 dark:border-chalk/20 py-1 font-semibold transition-colors hover:border-pop-pink hover:text-pop-pink"
				>
					switch
				</button>
			</div>
		</header>
	{/if}
	{@render children()}
</div>
