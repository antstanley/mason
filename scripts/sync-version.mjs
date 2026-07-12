// mason ships as one thing, so it has one version. Changesets owns it, in the
// root package.json; this propagates that number everywhere else it appears.
// Runs as part of `pnpm version`, right after `changeset version`.
//
// Before changesets, the repo held three versions that all disagreed:
// web/package.json 0.0.1, the Cargo workspace 0.1.0, and the v0.1.0 release.
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const version = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')).version;

/** the web app: keep its package version in step (it is private, never published) */
const webPath = resolve(root, 'web/package.json');
const web = JSON.parse(readFileSync(webPath, 'utf8'));
if (web.version !== version) {
	web.version = version;
	writeFileSync(webPath, `${JSON.stringify(web, null, '\t')}\n`);
	console.log(`web/package.json      -> ${version}`);
}

/** the Rust workspace: [workspace.package] version, inherited by every crate */
const cargoPath = resolve(root, 'server/Cargo.toml');
const cargo = readFileSync(cargoPath, 'utf8');
const bumped = cargo.replace(
	/(\[workspace\.package\][^[]*?\nversion = ")[^"]+(")/,
	`$1${version}$2`
);
if (bumped !== cargo) {
	writeFileSync(cargoPath, bumped);
	console.log(`server/Cargo.toml     -> ${version}`);
}

/** Cargo.lock is tracked and records the workspace crates' own versions, so it
 *  has to follow or `cargo build --locked` fails on a stale lock. Patch it here
 *  rather than requiring a Rust toolchain in the release job. */
const lockPath = resolve(root, 'server/Cargo.lock');
const lock = readFileSync(lockPath, 'utf8');
const relocked = lock.replace(
	/(\[\[package\]\]\nname = "mortar-(?:core|server|wasm)"\nversion = ")[^"]+(")/g,
	`$1${version}$2`
);
if (relocked !== lock) {
	writeFileSync(lockPath, relocked);
	console.log(`server/Cargo.lock     -> ${version}`);
}

console.log(`\nmason is ${version}`);
