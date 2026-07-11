<script lang="ts">
	import { goto } from '$app/navigation';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';

	let value = $state(lastHandle.value);

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(value);
		if (!handle) return;
		lastHandle.remember(handle);
		void goto(`/?actor=${encodeURIComponent(handle)}`);
	}
</script>

<div class="mx-auto flex min-h-[70vh] max-w-xl flex-col items-center justify-center gap-8 text-center">
	<div class="flex gap-2 text-4xl" aria-hidden="true">
		<span class="rotate-[-6deg] rounded-lg bg-brick-post px-3 py-1.5 shadow-brick">🦋</span>
		<span class="rotate-[3deg] rounded-lg bg-brick-blog px-3 py-1.5 shadow-brick">📝</span>
		<span class="rotate-[-2deg] rounded-lg bg-brick-video px-3 py-1.5 shadow-brick">🎬</span>
	</div>
	<div>
		<h1 class="font-display text-6xl font-black tracking-tight">mason</h1>
		<p class="mt-3 text-lg opacity-75">
			one wall, every brick — posts, blogs &amp; trailers from the people you follow
		</p>
	</div>
	<form onsubmit={submit} class="flex w-full max-w-md gap-2">
		<input
			bind:value
			type="text"
			placeholder="your.handle.bsky.social"
			autocapitalize="none"
			autocorrect="off"
			spellcheck="false"
			class="min-w-0 flex-1 rounded-full border-2 border-ink/20 bg-chalk px-5 dark:border-chalk/20 dark:bg-kiln py-3 font-semibold outline-none transition-colors focus:border-pop-pink"
		/>
		<button
			type="submit"
			class="cursor-pointer rounded-full bg-pop-pink px-6 py-3 font-display font-bold text-white shadow-brick transition-transform hover:scale-105 active:scale-95"
		>
			lay bricks
		</button>
	</form>
	<p class="text-sm opacity-50">no login — we just peek at your public follows</p>
</div>
