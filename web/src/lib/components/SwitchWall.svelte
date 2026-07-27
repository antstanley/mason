<script lang="ts">
	// Whoever's wall this is, their face doubles as a switcher. Clicking it does
	// NOT leave the current wall; it drops a small form below the button. Nothing
	// re-renders until a DIFFERENT handle is submitted, so opening the panel and
	// thinking better of it leaves the wall exactly where it was.
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { feedInfo } from '$lib/state/feedinfo.svelte';
	import { feeds } from '$lib/state/feeds.svelte';
	import { cleanHandle, lastHandle } from '$lib/state/handle.svelte';
	import { profile } from '$lib/state/profile.svelte';
	import Icon from './Icon.svelte';

	// Both spellings of a wall, and neither is required: the header takes what
	// the URL has. A graph wall shows the owner's face, a feed wall shows the
	// generator's, and with both parameters present the feed is what is laid,
	// so the feed is what the button names.
	let { actor, feed }: { actor: string | null; feed: string | null } = $props();

	let open = $state(false);
	let root = $state<HTMLElement | null>(null);
	let trigger = $state<HTMLButtonElement | null>(null);
	let input = $state<HTMLInputElement | null>(null);
	let value = $state('');

	const onFeed = $derived(!!feed);

	// the face on the button, from whichever identity this wall has. Neither
	// load blocks anything: both leave a fallback in place and fill it in if and
	// when the AppView answers.
	$effect(() => {
		if (feed) {
			feedInfo.load(feed);
		} else {
			profile.load(actor ?? '');
		}
	});

	// what the button says it is showing, and the initial it falls back to when
	// there is no face yet
	const wallName = $derived(onFeed ? feedInfo.name : `@${actor ?? ''}`);
	const wallFace = $derived(onFeed ? feedInfo.avatar : profile.avatar);
	// A feed's own name is not unique (two "Discover" feeds are ordinary), so the
	// creator's handle is what tells a screen-reader reader which one this is.
	// Before that lands, wallName is the reference's rkey, which is at least
	// something the reader can match against the link they followed.
	const switchLabel = $derived(
		onFeed
			? `Switch wall, currently viewing the ${wallName} feed${feedInfo.creator ? ` by @${feedInfo.creator}` : ''}`
			: `Switch wall, currently viewing ${wallName}`
	);

	function openPanel() {
		open = true;
		// prefill the reader's own remembered handle, so switching back to your
		// own wall is a single tap; select it so typing a new one just replaces
		value = lastHandle.value;
		void tick().then(() => {
			input?.focus();
			input?.select();
		});
	}

	function closePanel(returnFocus = true) {
		open = false;
		if (returnFocus) trigger?.focus();
	}

	// Tab out of the dialog and it closes, so focus never walks into the dimmed
	// wall behind (mirrors the Escape and click-away dismissals). Only a move that
	// lands outside the whole switcher counts; hops between the fields do not.
	function onFocusOut(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (next && root && !root.contains(next)) closePanel(false);
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const handle = cleanHandle(value);
		if (!handle) return;
		open = false;
		// same wall: close and change nothing, so the page never re-renders on a
		// no-op switch
		if (handle === actor) return;
		lastHandle.remember(handle);
		void goto(`/?actor=${encodeURIComponent(handle)}`);
	}

	$effect(() => {
		if (!open) return;
		const onDown = (event: PointerEvent) => {
			if (root && !root.contains(event.target as Node)) closePanel(false);
		};
		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') closePanel();
		};
		document.addEventListener('pointerdown', onDown);
		document.addEventListener('keydown', onKey);
		return () => {
			document.removeEventListener('pointerdown', onDown);
			document.removeEventListener('keydown', onKey);
		};
	});
</script>

<div bind:this={root} class="relative">
	<button
		bind:this={trigger}
		type="button"
		onclick={() => (open ? closePanel(false) : openPanel())}
		aria-haspopup="dialog"
		aria-expanded={open}
		aria-label={switchLabel}
		class="inline-flex min-h-11 min-w-11 cursor-pointer items-center justify-center overflow-hidden rounded-full bg-pop-pink-quiet p-0.5 font-semibold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95 sm:min-h-9 sm:min-w-0 sm:justify-start sm:gap-2 sm:pr-4"
	>
		{#if wallFace}
			<img src={wallFace} alt="" class="size-8 shrink-0 rounded-full object-cover" />
		{:else}
			<span
				class="grid size-8 shrink-0 place-items-center rounded-full bg-white/20 text-base font-bold uppercase"
				aria-hidden="true"
			>
				{(onFeed ? feedInfo.name : (actor ?? '?')).slice(0, 1) || '?'}
			</span>
		{/if}
		<!-- the name is already spelled with its @ on a graph wall; a feed's is a
		     display name, and prefixing one would read as a handle -->
		<span class="hidden max-w-[10rem] truncate sm:inline">{wallName}</span>
		<Icon name="arrow-left-right" class="hidden size-4 shrink-0 opacity-80 sm:block" />
	</button>

	{#if open}
		<!-- a filter over the wall: dims it in light mode, veils it in dark, and
		     swallows every click so the content behind cannot be touched while the
		     switcher is up. tapping it is a dismiss. -->
		<button
			type="button"
			tabindex="-1"
			aria-label="Close switch panel"
			onclick={() => closePanel()}
			class="fixed inset-0 z-30 cursor-default bg-ink/35 backdrop-blur-[2px] dark:bg-chalk/15"
		></button>
		<div
			role="dialog"
			aria-modal="true"
			aria-label="Switch wall"
			onfocusout={onFocusOut}
			class="absolute right-0 bottom-full z-40 mb-2 w-80 max-w-[calc(100vw-2rem)] rounded-2xl border border-ink/10 bg-chalk p-5 text-left shadow-brick-lift md:top-full md:bottom-auto md:mt-2 md:mb-0 dark:border-chalk/15 dark:bg-kiln"
		>
			<form onsubmit={submit} class="flex flex-col gap-4">
				<label class="text-xs font-semibold opacity-75" for="switch-handle">switch to a handle</label>
				<input
					id="switch-handle"
					bind:this={input}
					bind:value
					type="text"
					placeholder="your.handle.bsky.social"
					autocapitalize="none"
					autocorrect="off"
					spellcheck="false"
					class="min-w-0 rounded-full border border-ink/15 bg-chalk px-4 py-2.5 text-sm font-semibold transition-colors focus:border-pop-pink focus-visible:outline-2 focus-visible:outline-offset-0 dark:border-chalk/15 dark:bg-kiln"
				/>
				<button
					type="submit"
					class="cursor-pointer rounded-full bg-pop-pink-deep px-4 py-2.5 font-display text-sm font-bold text-white shadow-brick transition-transform motion-safe:hover:scale-105 motion-safe:active:scale-95"
				>
					lay bricks
				</button>
				{#if actor !== 'demo'}
					<a
						href="/?actor=demo"
						class="text-center text-xs font-semibold text-brick-post-ink hover:underline dark:text-brick-post"
					>
						or wander the demo wall
					</a>
				{/if}
			</form>
			<!-- mason's other front door, from a laid wall: this panel is already the
			     switch-walls affordance, so the feed picker belongs on it rather than
			     beside it. The panel closes FIRST, which hands focus back to the
			     switcher's own button: this control unmounts with the panel, and the
			     picker returns focus to whatever held it, so that has to be something
			     still on the page. -->
			<button
				type="button"
				onclick={() => {
					closePanel();
					feeds.openPicker();
				}}
				class="mt-4 inline-flex min-h-11 w-full cursor-pointer items-center justify-center rounded-full border border-ink/15 px-4 text-sm font-semibold transition-colors hover:bg-ink/5 dark:border-chalk/15 dark:hover:bg-chalk/10"
			>
				or pick a feed to lay
			</button>
		</div>
	{/if}
</div>
