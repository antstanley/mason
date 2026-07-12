<script lang="ts">
	import { goto } from '$app/navigation';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';
	import LandingWall from './LandingWall.svelte';

	let value = $state(lastHandle.value);
	let input = $state<HTMLInputElement | null>(null);

	// the landing has one job — the first keystroke should land in it
	$effect(() => input?.focus());

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(value);
		if (!handle) return;
		lastHandle.remember(handle);
		void goto(`/?actor=${encodeURIComponent(handle)}`);
	}
</script>

<main class="relative isolate flex min-h-screen flex-col items-center justify-center py-16">
	<LandingWall />

	<div
		class="mx-auto flex w-full max-w-lg flex-col items-center gap-7 rounded-card border-2 border-ink/10 bg-plaster/85 p-8 text-center shadow-brick-lift backdrop-blur-sm sm:p-10 dark:border-chalk/10 dark:bg-kiln-deep/85"
	>
		<div>
			<h1 class="font-display text-6xl font-black tracking-tight">mason</h1>
			<p class="mt-3 text-lg text-balance opacity-80">
				the atmosphere, browsable — posts, blogs and video from everyone you follow, in one wall
			</p>
		</div>

		<form onsubmit={submit} class="flex w-full flex-col gap-2 sm:flex-row">
			<label class="sr-only" for="handle">Your Bluesky handle</label>
			<input
				id="handle"
				bind:this={input}
				bind:value
				type="text"
				placeholder="your.handle.bsky.social"
				autocapitalize="none"
				autocorrect="off"
				spellcheck="false"
				class="min-w-0 flex-1 rounded-full border-2 border-ink/20 bg-chalk px-5 py-3 font-semibold transition-colors focus:border-pop-pink dark:border-chalk/20 dark:bg-kiln"
			/>
			<button
				type="submit"
				class="shrink-0 cursor-pointer rounded-full bg-pop-pink-deep px-6 py-3 max-sm:w-full font-display font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
			>
				lay bricks
			</button>
		</form>

		<div class="flex flex-col items-center gap-2 text-sm">
			<p class="opacity-75">
				no login — mason just reads your public follows, right here in your browser
			</p>
			<a
				href="/?actor=demo"
				class="inline-flex min-h-11 items-center px-2 font-semibold text-brick-post-ink hover:underline dark:text-brick-post"
			>
				don't have one? wander the demo wall →
			</a>
		</div>
	</div>
</main>
