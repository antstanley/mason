// Exactly one video plays at a time: mounting a player claims the slot,
// which reactively tears down whichever player held it before.
class PlayerState {
	activeId = $state<string | null>(null);

	claim(id: string) {
		this.activeId = id;
	}

	release(id: string) {
		if (this.activeId === id) this.activeId = null;
	}
}

export const player = new PlayerState();
