<script lang="ts">
	// The layout toggle has two shapes, and which one shows is a width decision.
	//
	// From `sm` up it is a slider: one thumb slides across the options and the
	// labels crossfade as it passes under them. The segments are sized to their
	// own content (labels differ in length), so the thumb measures the selected
	// label and matches its position and width, so it hugs each option instead of
	// leaving dead space around the short ones. Native radios underneath keep it
	// keyboard- and screen-reader-honest (arrow keys move the selection), and the
	// thumb holds still under prefers-reduced-motion.
	//
	// Below `sm` it is a dropdown, in the same language as `ClientPicker`. Three
	// segments laid side by side is the widest thing on a bar that never wraps,
	// and it was wide enough to push two of its own touch targets under 44px to
	// make room for a fourth control. A trigger plus a popover spends one target's
	// width instead of three and gives the shaved pixels back.
	//
	// Two shapes rather than one, and both are in the markup with the other
	// `display: none`, which takes it out of the tab order and the accessibility
	// tree alike. The mobile shape is a listbox of buttons rather than a second
	// set of radios on purpose: two radio groups sharing a `name` are one group to
	// the browser however they are styled, so checking one would quietly uncheck
	// the other.
	import { tick } from 'svelte';
	import { LAYOUTS, layout, type LayoutId } from '$lib/state/layout.svelte';
	import Icon from './Icon.svelte';

	let open = $state(false);
	let root = $state<HTMLElement | null>(null);
	let trigger = $state<HTMLButtonElement | null>(null);
	let listbox = $state<HTMLElement | null>(null);

	const current = $derived(LAYOUTS.find((option) => option.id === layout.id) ?? LAYOUTS[0]);

	function rows(): HTMLElement[] {
		return listbox ? Array.from(listbox.querySelectorAll<HTMLElement>('[role="option"]')) : [];
	}

	function openMenu() {
		open = true;
		void tick().then(() => {
			const opts = rows();
			opts[Math.max(0, selected)]?.focus();
		});
	}

	function closeMenu(returnFocus = true) {
		open = false;
		if (returnFocus) trigger?.focus();
	}

	function choose(id: LayoutId) {
		layout.set(id);
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

	const selected = $derived(Math.max(
		0,
		LAYOUTS.findIndex((option) => option.id === layout.id)
	));

	let track = $state<HTMLElement | null>(null);
	let x = $state(0);
	let w = $state(0);

	function measure() {
		// read `selected` unconditionally so the effect below tracks it and
		// re-measures when the selection moves
		const i = selected;
		const el = track?.querySelectorAll<HTMLElement>('label')[i];
		if (!el) return;
		x = el.offsetLeft;
		w = el.offsetWidth;
	}

	$effect(() => {
		measure();
		if (!track) return;
		// labels reflow as the viewport changes (and once the emoji fonts land)
		const observer = new ResizeObserver(measure);
		observer.observe(track);
		return () => observer.disconnect();
	});
</script>

<!-- below sm: the trigger and its popover -->
<div bind:this={root} class="relative sm:hidden">
	<button
		bind:this={trigger}
		type="button"
		onclick={() => (open ? closeMenu(false) : openMenu())}
		onkeydown={onTriggerKey}
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-label="Wall layout, currently {current.label}"
		class="flex min-h-11 items-center gap-1.5 rounded-full border-2 border-ink/15 px-2.5 text-sm font-semibold transition-colors hover:bg-ink/5 dark:border-chalk/20 dark:hover:bg-chalk/10"
	>
		<span aria-hidden="true" class="text-lg leading-none">{current.icon}</span>
		<!-- No caption over the name: the icon and the layout's own name carry it,
		     and the word was spending width on a bar that has none to spare. The
		     accessible name still says what this control is, since "Bento" alone
		     would tell a screen reader nothing.

		     The names are stacked in one grid cell so the trigger is as wide as the
		     longest of them and the bar does not shift as the layout changes. -->
		<span class="grid text-left text-sm">
			{#each LAYOUTS as option (option.id)}
				<span class="col-start-1 row-start-1 {option.id === current.id ? '' : 'invisible'}">
					{option.label}
				</span>
			{/each}
		</span>
		<span
			aria-hidden="true"
			class="opacity-60 transition-transform duration-200 {open ? 'rotate-180' : ''}"
		>
			<Icon name="chevron-down" class="size-3.5" />
		</span>
	</button>

	{#if open}
		<ul
			bind:this={listbox}
			role="listbox"
			aria-label="Wall layout"
			onkeydown={onListKey}
			class="absolute bottom-full left-0 z-20 mb-2 min-w-full overflow-hidden rounded-2xl border-2 border-ink/10 bg-chalk p-1 shadow-brick-lift dark:border-chalk/15 dark:bg-kiln"
		>
			{#each LAYOUTS as option (option.id)}
				<li>
					<button
						type="button"
						role="option"
						aria-selected={option.id === layout.id}
						tabindex="-1"
						onclick={() => choose(option.id as LayoutId)}
						class="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm font-semibold whitespace-nowrap transition-colors hover:bg-ink/5 aria-selected:bg-ink/[0.06] dark:hover:bg-chalk/10 dark:aria-selected:bg-chalk/10"
					>
						<span aria-hidden="true" class="text-base leading-none">{option.icon}</span>
						<span class="flex-1">{option.label}</span>
						<span
							aria-hidden="true"
							class="shrink-0 text-brick-post-ink dark:text-brick-post {option.id === layout.id
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

<!-- from sm: the slider -->
<fieldset class="hidden rounded-full border-2 border-ink/15 p-1 sm:block dark:border-chalk/20">
	<legend class="sr-only">Wall layout</legend>
	<div bind:this={track} class="relative flex">
		<span
			aria-hidden="true"
			class="pointer-events-none absolute inset-y-0 left-0 rounded-full bg-ink shadow-brick transition-[transform,width] duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] motion-reduce:transition-none dark:bg-chalk"
			style:width="{w}px"
			style:transform="translateX({x}px)"
		></span>
		{#each LAYOUTS as option (option.id)}
			<label class="relative z-10 cursor-pointer">
				<input
					type="radio"
					name="layout"
					value={option.id}
					checked={layout.id === option.id}
					onchange={() => layout.set(option.id as LayoutId)}
					class="peer sr-only"
				/>
				<!-- this shape only ever renders from sm, so it can spend the width the
				     dropdown above exists to save below it: the label sits beside the
				     icon rather than under it, with room around both. -->
				<span
					class="flex min-h-9 flex-row items-center justify-center gap-1.5 rounded-full px-4 text-sm font-semibold text-ink transition-colors duration-300 peer-checked:text-chalk peer-focus-visible:outline-3 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-pop-pink-deep motion-reduce:transition-none dark:text-chalk dark:peer-checked:text-ink"
				>
					<span aria-hidden="true" class="text-sm leading-none">{option.icon}</span>
					<span class="text-sm">{option.label}</span>
				</span>
			</label>
		{/each}
	</div>
</fieldset>
