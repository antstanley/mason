import { browser } from "$app/environment";

const STORAGE_KEY = "mason:handle";

class HandleState {
  current = $state<string | null>(browser ? localStorage.getItem(STORAGE_KEY) : null);

  set(raw: string) {
    const cleaned = raw.trim().replace(/^@/, "").toLowerCase();
    if (!cleaned) return;
    this.current = cleaned;
    if (browser) localStorage.setItem(STORAGE_KEY, cleaned);
  }

  clear() {
    this.current = null;
    if (browser) localStorage.removeItem(STORAGE_KEY);
  }
}

export const handle = new HandleState();
