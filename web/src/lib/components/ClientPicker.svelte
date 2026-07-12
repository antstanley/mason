<script lang="ts">
	// Which atmosphere client should a brick open in? A native <select>: it is
	// one setting, it belongs in the chrome, and reinventing it as a custom
	// dropdown would buy nothing but bugs.
	import { CLIENTS, client, type ClientId } from '$lib/state/client.svelte';
</script>

<label class="flex items-center gap-2">
	<span class="sr-only">Open posts in</span>
	<span class="text-sm opacity-75" aria-hidden="true">open in</span>
	<select
		value={client.id}
		onchange={(event) => client.set(event.currentTarget.value as ClientId)}
		class="min-h-11 cursor-pointer rounded-full border-2 border-ink/15 bg-chalk px-3 text-sm font-semibold transition-colors hover:border-pop-pink dark:border-chalk/20 dark:bg-kiln"
	>
		{#each CLIENTS as option (option.id)}
			<option value={option.id}>{option.label}</option>
		{/each}
	</select>
</label>
