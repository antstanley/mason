<script lang="ts">
	// Which atmosphere client should a brick open in? Icons live inside the
	// options, which a native <select> can't render, so this is a custom listbox:
	// a pill trigger and a popover of rows, with roving arrow-key focus, Escape,
	// and click-away, so it stays as keyboard-honest as the select it replaces.
	import { tick } from 'svelte';
	import { CLIENTS, client, type ClientId } from '$lib/state/client.svelte';
	import ClientIcon from './ClientIcon.svelte';
	import Icon from './Icon.svelte';

	let open = $state(false);
	let root = $state<HTMLElement | null>(null);
	let trigger = $state<HTMLButtonElement | null>(null);
	let listbox = $state<HTMLElement | null>(null);

	const current = $derived(CLIENTS.find((c) => c.id === client.id) ?? CLIENTS[0]);

	function rows(): HTMLElement[] {
		return listbox ? Array.from(listbox.querySelectorAll<HTMLElement>('[role="option"]')) : [];
	}

	function openMenu() {
		open = true;
		void tick().then(() => {
			const opts = rows();
			const idx = CLIENTS.findIndex((c) => c.id === client.id);
			opts[Math.max(0, idx)]?.focus();
		});
	}

	function closeMenu(returnFocus = true) {
		open = false;
		if (returnFocus) trigger?.focus();
	}

	function choose(id: ClientId) {
		client.set(id);
		closeMenu();
	}

	function onTriggerKey(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			openMenu();
		}
	}

	function onListKey(event: KeyboardEvent) {
		const opts = rows();
		const i = opts.indexOf(document.activeElement as HTMLElement);
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			opts[(i + 1) % opts.length]?.focus();
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			opts[(i - 1 + opts.length) % opts.length]?.focus();
		} else if (event.key === 'Home') {
			event.preventDefault();
			opts[0]?.focus();
		} else if (event.key === 'End') {
			event.preventDefault();
			opts[opts.length - 1]?.focus();
		} else if (event.key === 'Escape' || event.key === 'Tab') {
			closeMenu(event.key === 'Escape');
		}
	}

	$effect(() => {
		if (!open) return;
		const onDown = (event: PointerEvent) => {
			if (root && !root.contains(event.target as Node)) closeMenu(false);
		};
		document.addEventListener('pointerdown', onDown);
		return () => document.removeEventListener('pointerdown', onDown);
	});
</script>

<div bind:this={root} class="relative">
	<button
		bind:this={trigger}
		type="button"
		onclick={() => (open ? closeMenu(false) : openMenu())}
		onkeydown={onTriggerKey}
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-label="Open posts in {current.label}"
		class="flex min-h-11 items-center gap-1.5 rounded-full px-2 text-sm font-semibold transition-colors hover:bg-ink/5 sm:px-3 dark:hover:bg-chalk/10"
	>
		<ClientIcon id={current.id} size="size-6 sm:size-[1.3em]" />
		<!-- a bare butterfly says nothing to a first-time visitor, so mobile shows
		     a tiny "opens in" caption over the client name; the chevron returns at
		     sm. Stacking every label in one cell keeps the trigger as wide as the
		     longest name so the header does not shift as clients change. -->
		<span class="flex flex-col text-left text-xs sm:block sm:text-sm">
			<span class="text-[0.625rem] leading-none opacity-75 sm:hidden">opens in</span>
			<span class="grid">
				{#each CLIENTS as option (option.id)}
					<span class="col-start-1 row-start-1 {option.id === current.id ? '' : 'invisible'}">
						{option.label}
					</span>
				{/each}
			</span>
		</span>
		<span
			aria-hidden="true"
			class="hidden opacity-60 transition-transform duration-200 sm:inline-block {open
				? 'rotate-180'
				: ''}"
		>
			<Icon name="chevron-down" class="size-3.5" />
		</span>
	</button>

	{#if open}
		<ul
			bind:this={listbox}
			role="listbox"
			aria-label="Open posts in"
			onkeydown={onListKey}
			class="absolute right-0 bottom-full z-20 mb-2 min-w-full overflow-hidden rounded-2xl border-2 border-ink/10 bg-chalk p-1 shadow-brick-lift md:top-full md:bottom-auto md:mt-2 md:mb-0 dark:border-chalk/15 dark:bg-kiln"
		>
			{#each CLIENTS as option (option.id)}
				<li>
					<button
						type="button"
						role="option"
						aria-selected={option.id === client.id}
						tabindex="-1"
						onclick={() => choose(option.id)}
						class="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm font-semibold whitespace-nowrap transition-colors hover:bg-ink/5 aria-selected:bg-ink/[0.06] dark:hover:bg-chalk/10 dark:aria-selected:bg-chalk/10"
					>
						<ClientIcon id={option.id} dense />
						<span class="flex-1">{option.label}</span>
						<span
							aria-hidden="true"
							class="shrink-0 text-brick-post-ink dark:text-brick-post {option.id === client.id
								? ''
								: 'invisible'}"
						>
							<Icon name="check" class="size-4" />
						</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>
