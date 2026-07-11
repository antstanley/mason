<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import { localMode } from '$lib/api';

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
	<title>{actor ? `@${actor} · mason` : 'mason — an atproto discovery wall'}</title>
	<link rel="icon" href={favicon} />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:wght@400;700;800&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="mx-auto min-h-screen max-w-[1800px] px-4 sm:px-6">
	{#if actor}
		<header class="flex items-center justify-between py-4">
			<a href="/" class="font-display text-2xl font-black tracking-tight">
				mason <span class="text-pop-pink">🧱</span>
			</a>
			<div class="flex items-center gap-3 text-sm">
				<span class="font-semibold opacity-70">@{actor}</span>
				<button
					type="button"
					onclick={() => void goto('/')}
					class="cursor-pointer rounded-full border-2 border-ink/15 px-3 dark:border-chalk/20 py-1 font-semibold transition-colors hover:border-pop-pink hover:text-pop-pink"
				>
					switch
				</button>
			</div>
		</header>
	{/if}
	{@render children()}
</div>
