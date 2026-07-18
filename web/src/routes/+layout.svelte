<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { localMode } from '$lib/api';
	import ClientPicker from '$lib/components/ClientPicker.svelte';
	import LayoutPicker from '$lib/components/LayoutPicker.svelte';
	import SwitchWall from '$lib/components/SwitchWall.svelte';

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

<div class="mx-auto min-h-screen max-w-[1800px] px-4 sm:px-6 {actor ? 'pb-24 md:pb-0' : ''}">
	{#if actor}
		<a
			href="#wall"
			class="sr-only focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-50 focus:rounded-full focus:bg-chalk focus:px-4 focus:py-2 focus:font-semibold dark:focus:bg-kiln"
		>
			skip to the wall
		</a>
		<header
			class="fixed inset-x-0 bottom-0 z-20 flex flex-col gap-2 border-t border-ink/10 bg-plaster/95 px-4 pt-2 pb-[calc(0.5rem+env(safe-area-inset-bottom))] md:static md:z-auto md:flex-row md:flex-wrap md:items-center md:justify-between md:border-0 md:bg-transparent md:px-0 md:py-3 dark:border-chalk/10 dark:bg-kiln-deep/95 md:dark:bg-transparent"
		>
			<a href="/" class="hidden min-h-11 items-center font-display text-2xl font-black tracking-tight md:inline-flex">
				mason&nbsp;<span aria-hidden="true">🧱</span>
			</a>
			<div class="flex flex-wrap items-center justify-between gap-3 text-sm md:justify-end">
				<LayoutPicker />
				<ClientPicker />
				<SwitchWall actor={actor ?? ''} />
			</div>
		</header>
	{/if}
	{@render children()}
</div>
