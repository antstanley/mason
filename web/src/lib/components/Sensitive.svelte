<script lang="ts">
	// Covers a brick's media behind a reveal when a !warn label rides on it.
	// mason is a logged-out reader, so this mirrors what Bluesky shows a
	// logged-out viewer: hard-hidden and adult media never reach the wall at
	// all (the engine drops them), so anything that gets here is the soft-warn
	// tier and can always be revealed. The choice is per brick and forgotten on
	// reload, by design: no storage, no lingering "show everything" switch.
	import type { Snippet } from 'svelte';
	import type { Blur } from '$lib/types';

	let { blur, children }: { blur?: Blur; children: Snippet } = $props();

	let revealed = $state(false);
</script>

{#if blur && !revealed}
	<div class="relative overflow-hidden">
		<div
			class="pointer-events-none scale-105 select-none blur-2xl [&_img]:blur-2xl"
			aria-hidden="true"
		>
			{@render children()}
		</div>
		<div class="absolute inset-0 grid place-items-center bg-ink/45 p-4 text-center dark:bg-kiln/55">
			<div class="flex flex-col items-center gap-2">
				<span class="text-2xl" aria-hidden="true">🫣</span>
				<p class="text-sm font-semibold text-chalk drop-shadow">sensitive media</p>
				<button
					type="button"
					onclick={() => (revealed = true)}
					aria-label="Show sensitive media"
					class="motion-safe:hover:scale-105 motion-safe:active:scale-95 cursor-pointer rounded-full bg-chalk/95 px-4 py-1.5 font-display text-sm font-bold text-ink shadow-brick transition-transform"
				>
					show anyway
				</button>
			</div>
		</div>
	</div>
{:else}
	{@render children()}
{/if}
