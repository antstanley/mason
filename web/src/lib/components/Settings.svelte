<script lang="ts">
	// The settings screen: the preferences a reader sets once and forgets.
	//
	// It is mounted as a SIBLING of the layout's content wrapper, never inside it,
	// for the same reason BrickReader and FeedPicker are: that wrapper goes
	// `inert` while any overlay is up and `inert` covers every descendant, so a
	// screen nested in the thing it dims would open unfocusable and invisible to
	// assistive tech.
	//
	// Every decision about whether it is open lives in `settings.svelte.ts`, which
	// vitest runs for real. What is left here is the shell, and the shell is the
	// half no lane in this repo can see.
	import { tick } from 'svelte';
	import { settings } from '$lib/state/settings.svelte';
	import ClientPicker from './ClientPicker.svelte';
	import Icon from './Icon.svelte';

	const isOpen = $derived(settings.isOpen);

	const uid = $props.id();
	const titleId = `${uid}-title`;

	function close() {
		settings.closeSettings();
	}

	let panel = $state<HTMLElement | null>(null);

	// Escape closes, in the same language as the reader's and the picker's. Bound
	// to the document so it works wherever focus has landed inside the screen, and
	// attached only while it is up (below), so it can never reach one already shut.
	const onKey = (event: KeyboardEvent) => {
		if (event.key === 'Escape') close();
	};

	// Everything the screen has to undo, keyed on `settings.isOpen` and nothing
	// else: the back gesture shuts it without ever calling close(), so the
	// teardown hangs off the state rather than off a control.
	$effect(() => {
		if (!isOpen) return;
		// captured BEFORE anything below moves focus, which is why the capture and
		// the focus live in one effect rather than two: two would run in whatever
		// order they were created, and one that focused the panel first would leave
		// this holding the panel as the element to hand focus back to.
		const active = document.activeElement;
		const opener = active instanceof HTMLElement ? active : null;
		const root = document.documentElement;
		const restore = root.style.overflow;
		// the page must not scroll behind the screen; the panel does its own
		root.style.overflow = 'hidden';
		document.addEventListener('keydown', onKey);
		// into the panel rather than into the first control: a settings screen is
		// something you read before you change, so a keyboard reader lands on the
		// heading rather than already inside a listbox. After a tick, because the
		// binding is set as the DOM updates.
		void tick().then(() => panel?.focus());
		return () => {
			root.style.overflow = restore;
			document.removeEventListener('keydown', onKey);
			// back to the cog that opened it, which stays on the bar while the
			// screen is up, so this lands somewhere a reader can carry on from
			opener?.focus();
		};
	});
</script>

{#if isOpen}
	<!-- the scrim is a real button, so a click anywhere off the panel closes it and
	     a screen reader is told what that does, in the same language as the
	     reader's and the picker's. -->
	<button
		type="button"
		tabindex="-1"
		aria-label="Close settings"
		onclick={close}
		class="fixed inset-0 z-40 animate-scrim-in cursor-default bg-ink/50 backdrop-blur-[2px] dark:bg-kiln-deep/70"
	></button>
	<!-- pointer-events-none, so the gutter around the panel is the scrim rather
	     than a dead zone: a click beside the panel lands on the button under it -->
	<!-- The panel is NOT a scroll container, which is load-bearing rather than an
	     omission. A scroll container clips and scrolls its absolutely positioned
	     descendants, so the client combobox's popover extended this dialog's
	     scroll height from 255px to 431px the moment it opened: a scrollbar
	     appeared inside the modal and the list was trapped behind it. The panel
	     holds one setting and is ~257px tall, so it fits any viewport mason
	     supports with room to spare. If settings ever grows past the fold, the
	     scroll region belongs on an inner wrapper and the combobox needs
	     positioning that escapes it; a scroll container around the popover does
	     not. -->
	<div class="pointer-events-none fixed inset-0 z-50 grid place-items-center overflow-y-auto p-3 sm:p-6">
		<div
			bind:this={panel}
			role="dialog"
			aria-modal="true"
			aria-labelledby={titleId}
			tabindex="-1"
			class="pointer-events-auto w-full max-w-md animate-reader-in rounded-card border border-ink/10 bg-chalk p-4 text-left shadow-brick-lift focus-visible:outline-none sm:p-6 dark:border-chalk/15 dark:bg-kiln"
		>
			<div class="flex items-start justify-between gap-3">
				<div>
					<h2 id={titleId} class="font-display text-xl font-black tracking-tight">settings</h2>
					<p class="mt-1 text-sm opacity-75">
						kept on this device, never on a wire. mason has no account to keep them in.
					</p>
				</div>
				<button
					type="button"
					onclick={close}
					aria-label="Close settings"
					class="-mr-1 flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full transition-colors hover:bg-ink/5 dark:hover:bg-chalk/10"
				>
					<Icon name="x" class="size-5" />
				</button>
			</div>

			<div class="mt-5 border-t border-ink/10 pt-5 dark:border-chalk/10">
				<!-- The client picker lives here rather than on the bar: which app a post
				     opens in is a choice a reader makes once, and the bar is for what
				     changes while they read. Moving it here is what gave the layout
				     picker and the refresh control their width back on a phone. -->
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div>
						<h3 class="text-sm font-semibold">open posts in</h3>
						<p class="mt-0.5 text-xs opacity-75">
							which atmosphere client a brick's link goes to.
						</p>
					</div>
					<ClientPicker wide />
				</div>
			</div>
		</div>
	</div>
{/if}
