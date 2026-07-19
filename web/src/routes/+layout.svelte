<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import { beforeNavigate } from '$app/navigation';
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
	//
	// A deploy swaps the whole engine (new wasm hash), so a page must not keep
	// running an old worker against new assets. `updateViaCache: 'none'` makes
	// the browser always revalidate the worker script instead of trusting an
	// HTTP-cached copy, and `update()` forces that check on load.
	//
	// Deploy-reload policy (#36): when a NEW worker takes control (only after a
	// deploy, never on a first install) the page must eventually reload so page
	// and engine are the same version, but never out from under the user: a
	// hard reload mid-session drops the laid wall, the scroll position, and any
	// playing video. So the reload is deferred: flag it as pending, then apply
	// it when the user is not looking - the tab going hidden, or the next
	// client-side navigation (turned into a full-page load so page and engine
	// leave together), whichever comes first. A tab that is already hidden when
	// the worker flips reloads immediately.
	//
	// A tab that loaded uncontrolled splits into two states (see $lib/api.ts):
	// a true first install, where the one flip is the new worker adopting the
	// tab (not a deploy), and a shift-reload beside an already-active worker,
	// where the byte-identical script re-registers with no install/activate/
	// claim at all, so no adoption flip ever fires and the first flip this tab
	// sees IS a deploy. The registration path below discriminates the two.
	let pendingReload = false;

	beforeNavigate(({ willUnload, to }) => {
		if (!pendingReload || willUnload || !to?.url) return;
		// setting location.href aborts the client-side navigation and performs
		// a full-page load of the same destination on the new build
		location.href = to.url.href;
	});

	$effect(() => {
		if (!browser || !localMode || !('serviceWorker' in navigator)) return;
		const sw = navigator.serviceWorker;
		let hadController = !!sw.controller;
		const onControllerChange = () => {
			if (!hadController) {
				// the same-version worker adopting an uncontrolled tab, not a deploy
				hadController = true;
				return;
			}
			if (pendingReload) return;
			if (document.visibilityState === 'hidden') {
				location.reload();
				return;
			}
			pendingReload = true;
		};
		const onVisibilityChange = () => {
			if (pendingReload && document.visibilityState === 'hidden') location.reload();
		};
		sw.addEventListener('controllerchange', onControllerChange);
		document.addEventListener('visibilitychange', onVisibilityChange);
		void (async () => {
			try {
				if (!hadController && (await sw.getRegistration())?.active) {
					// uncontrolled, but a same-version worker is already active: the
					// shift-reload state. It claimed its clients long ago, so no
					// adoption flip is coming; any flip from here on is a deploy.
					// Checked before register() so the flag is set before our own
					// update check could activate a new worker; a deploy landing in
					// the sliver before this resolves would still read as adoption,
					// and the next deploy converges.
					hadController = true;
				}
				const registration = await sw.register('/service-worker.js', {
					type: 'module',
					updateViaCache: 'none',
				});
				await registration.update();
			} catch {
				// registration is best-effort; a failed register just means no offline
			}
		})();
		return () => {
			sw.removeEventListener('controllerchange', onControllerChange);
			document.removeEventListener('visibilitychange', onVisibilityChange);
		};
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
