<script lang="ts">
	// One brick, read in place, over the wall it came from. Mounted once in
	// +layout.svelte as a SIBLING of the content wrapper, never inside it: that
	// wrapper goes `inert` while the reader is up and `inert` covers every
	// descendant, so a reader nested in the thing it dims would open
	// unfocusable and invisible to assistive tech.
	//
	// Every decision the reader makes lives in the `reader` rune, which is
	// typechecked and unit tested; a .svelte body is neither. What is left here
	// is markup, focus, the scroll lock and one key listener.
	import { reader } from '$lib/state/reader.svelte';
	import AuthorChip from './AuthorChip.svelte';
	import Icon from './Icon.svelte';

	// The panel, focused as it opens so a keyboard reader lands inside the
	// reader rather than at the top of the inert wall behind it.
	let panel = $state<HTMLElement | null>(null);

	// What to render, straight off the rune. `showing` is history state AND the
	// held brick agreeing, which is the SAME predicate the layout hangs the
	// wall's `inert` and this file hangs the scroll lock on, read from one place
	// so the three cannot answer differently. Page state on its own is wider: an
	// entry outlives the rune that pushed it, so a reload then a forward
	// navigation restores an id with no brick behind it, and the answer to that
	// is a shut reader over a live wall.
	const brick = $derived(reader.showing);

	// The same answer as a boolean, read through a derived rather than off the
	// rune inside the effect below. A derived wakes its readers only when the
	// VALUE changes, and stepping replaces the history entry without changing
	// whether the reader is up. Read straight off the rune, the teardown below
	// runs on every step rather than once per read: it releases and retakes the
	// scroll lock and hands focus back to the card mid-read, which is survivable
	// today only because that card sits inside the inert wrapper and cannot take
	// focus. Stepping is task 05; this is here before it lands.
	const isOpen = $derived(reader.isOpen);

	// The dialog's accessible name comes off the brick itself: a blog leads with
	// its title, and every other kind leads with its author line, which is the
	// first thing the panel renders.
	const label = $derived.by(() => {
		const open = brick;
		if (!open) return '';
		return open.kind === 'blog' ? open.title : (open.author.displayName ?? `@${open.author.handle}`);
	});

	// Focus moves into the reader as it opens. The binding is the signal rather
	// than a flag: it is set the moment the panel enters the DOM and cleared as
	// it leaves, so this focuses on open and does nothing at all on close.
	$effect(() => {
		panel?.focus();
	});

	// Escape closes. Bound to the document rather than to the panel, so it works
	// wherever focus has landed inside the reader, and attached only while the
	// reader is up (below), so it can never close a reader that is already shut.
	const onKey = (event: KeyboardEvent) => {
		if (event.key === 'Escape') reader.close();
	};

	// Everything the reader has to undo, keyed on `reader.isOpen` and on nothing
	// else. The back gesture shuts the reader without ever calling close(), so
	// the teardown hangs off the state rather than off a control. It must never
	// call reader.close() itself, either: close() pops history only for an entry
	// it pushed and clears that flag only inside itself, so a close() on an
	// already-shut reader would pop a SECOND entry and leave the wall.
	$effect(() => {
		if (!isOpen) return;
		const root = document.documentElement;
		const restore = root.style.overflow;
		// the wall must not scroll behind the reader. This does not stop the
		// pump: bricks may still be appended back there, which is harmless,
		// since an append never moves a laid brick and the reader locates its
		// own brick by id.
		root.style.overflow = 'hidden';
		// on the document rather than the panel: Escape works wherever focus has
		// landed inside the reader, and this runs only while the reader is up
		document.addEventListener('keydown', onKey);
		return () => {
			root.style.overflow = restore;
			document.removeEventListener('keydown', onKey);
			// back where they were on the wall: the card this was opened from.
			// Runs on unmount too, which is the only other way teardown happens.
			reader.returnFocus();
		};
	});
</script>

{#if brick}
	<!-- the scrim: dims the wall in light mode, veils it in dark, and swallows
	     every click so nothing behind can be touched while the reader is up.
	     Tapping it is a dismiss, in the same language as SwitchWall's. -->
	<button
		type="button"
		tabindex="-1"
		aria-label="Close the reader"
		onclick={() => reader.close()}
		class="fixed inset-0 z-40 animate-scrim-in cursor-default bg-ink/50 backdrop-blur-[2px] dark:bg-kiln-deep/70"
	></button>
	<!-- pointer-events-none, so the gutter around the panel is the scrim rather
	     than a dead zone: a click beside the panel lands on the button under it -->
	<div class="pointer-events-none fixed inset-0 z-50 grid place-items-center p-3 sm:p-6">
		<!-- tabindex -1: the panel takes focus from script and never from a Tab,
		     which is why it drops the focus ring. Chromium draws one on a
		     programmatically focused container even when the open came from a
		     mouse, and a ring that marks nothing the keyboard can reach is noise. -->
		<div
			bind:this={panel}
			role="dialog"
			aria-modal="true"
			aria-label={label}
			tabindex="-1"
			class="pointer-events-auto max-h-full w-full max-w-2xl animate-reader-in overflow-y-auto overscroll-contain rounded-card border border-ink/10 bg-chalk p-4 text-left shadow-brick-lift focus-visible:outline-none sm:p-6 dark:border-chalk/15 dark:bg-kiln"
		>
			<div class="flex items-start justify-between gap-3">
				<AuthorChip author={brick.author} avatarClass="size-10" />
				<button
					type="button"
					onclick={() => reader.close()}
					aria-label="Close the reader"
					class="-mt-1 -mr-1 inline-flex min-h-11 min-w-11 shrink-0 cursor-pointer items-center justify-center rounded-full transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					<Icon name="x" class="size-5" />
				</button>
			</div>
		</div>
	</div>
{/if}
