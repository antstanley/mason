<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { localMode } from '$lib/api';
	import ClientPicker from '$lib/components/ClientPicker.svelte';

	let { children } = $props();

	const actor = $derived(page.url.searchParams.get('actor'));

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
	<title>{actor ? `@${actor} · mason` : 'mason · one wall, every brick'}</title>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:wght@400;700;800&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="mx-auto min-h-screen max-w-[1800px] px-4 sm:px-6">
	{#if actor}
		<a
			href="#wall"
			class="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-50 focus:rounded-full focus:bg-chalk focus:px-4 focus:py-2 focus:font-semibold dark:focus:bg-kiln"
		>
			skip to the wall
		</a>
		<header class="flex items-center justify-between py-3">
			<a href="/" class="inline-flex min-h-11 items-center font-display text-2xl font-black tracking-tight">
				mason&nbsp;<span aria-hidden="true">🧱</span>
			</a>
			<div class="flex items-center gap-3 text-sm">
				<ClientPicker />
				<span class="font-semibold opacity-75">@{actor}</span>
				<a
					href="/"
					class="inline-flex min-h-11 items-center rounded-full border-2 border-ink/15 px-4 font-semibold transition-colors hover:border-pop-pink hover:text-pop-pink dark:border-chalk/20"
				>
					switch
				</a>
			</div>
		</header>
	{/if}
	{@render children()}
</div>
