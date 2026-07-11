<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { feed } from '$lib/state/feed.svelte';
	import FeedGrid from '$lib/components/FeedGrid.svelte';
	import HandleForm from '$lib/components/HandleForm.svelte';

	// the URL is the source of truth: /?actor=handle — shareable walls
	const actor = $derived(page.url.searchParams.get('actor'));

	$effect(() => {
		const current = actor;
		// untrack: reset mutates feed state; tracking it would loop this effect
		if (current) untrack(() => feed.reset(current));
	});
</script>

{#if actor}
	<main class="pb-8">
		<FeedGrid />
	</main>
{:else}
	<HandleForm />
{/if}
