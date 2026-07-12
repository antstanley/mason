import { browser } from "$app/environment";

const STORAGE_KEY = "mason:handle";

/** The URL (?actor=) is the source of truth for whose wall is showing -
 *  this only remembers the last handle to prefill the landing form. */
class LastHandle {
  value = $state<string>(browser ? (localStorage.getItem(STORAGE_KEY) ?? "") : "");

  remember(handle: string) {
    this.value = handle;
    if (browser) localStorage.setItem(STORAGE_KEY, handle);
  }
}

export const lastHandle = new LastHandle();

/** Normalize user input: strip @, trim, lowercase. */
export function cleanHandle(raw: string): string {
  return raw.trim().replace(/^@/, "").toLowerCase();
}
