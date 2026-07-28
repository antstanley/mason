<script lang="ts">
	// Covers a brick's media behind a reveal when a !warn label rides on it.
	// mason is a logged-out reader, so this mirrors what Bluesky shows a
	// logged-out viewer: hard-hidden and adult media never reach the wall at
	// all (the engine drops them), so anything that gets here is the soft-warn
	// tier and can always be revealed. The choice is per brick, keyed by brick
	// id in the shared `revealed` set, so uncovering one here uncovers it
	// wherever that brick is rendered next. It is still forgotten on reload, by
	// design: the set is a rune, not storage, so there is no lingering "show
	// everything" switch.
	import type { Snippet } from 'svelte';
	import type { Blur } from '$lib/types';
	import { revealed } from '$lib/state/sensitive.svelte';

	// `id` is the brick's own id, and it is required: it is the key the reveal
	// is remembered against, so a missing one would quietly untie the choice
	// from the brick and cover it again on the next render.
	// `| undefined` on `blur` is explicit because exactOptionalPropertyTypes
	// tells apart a prop left off from one passed as undefined, and every caller
	// forwards an optional brick field, which is the second case. The wire types
	// keep the bare `blur?: Blur` on purpose; this boundary accepts both.
	let { id, blur, children }: { id: string; blur?: Blur | undefined; children: Snippet } =
		$props();
</script>

{#if blur && !revealed.has(id)}
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
					onclick={(event) => {
						// On two of the four call sites this button sits INSIDE the card's
						// own anchor, so a reveal has two ways of turning into a trip
						// somewhere, and they need stopping separately. The propagation
						// stop keeps the click from reaching a handler on that anchor,
						// which is where the brick reader is going to hang its activation.
						// The default stop keeps the anchor's own href from opening the
						// post in a new tab, which propagation cannot reach: the browser
						// runs that navigation after dispatch, gated on defaultPrevented
						// alone. Together they keep "show anyway" a reveal and only a
						// reveal.
						event.preventDefault();
						event.stopPropagation();
						revealed.add(id);
					}}
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
