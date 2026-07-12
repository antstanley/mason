// mason publishes nothing to npm, so "release" means: tag the version and cut a
// GitHub release from the changelog changesets just wrote.
//
// The changesets action calls this after a Version PR is merged, when the
// working tree is already at the new version with no changesets left pending.
import { execFileSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const { version } = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
const tag = `v${version}`;

const run = (cmd, args) => execFileSync(cmd, args, { cwd: root, encoding: 'utf8' }).trim();

const existing = run('git', ['tag', '--list', tag]);
if (existing) {
	console.log(`${tag} already exists, nothing to release`);
	process.exit(0);
}

/** The section changesets wrote for this version, used verbatim as the notes. */
function notesFor(v) {
	const path = resolve(root, 'CHANGELOG.md');
	if (!existsSync(path)) return `mason ${v}`;
	const body = readFileSync(path, 'utf8');
	const start = body.indexOf(`## ${v}`);
	if (start === -1) return `mason ${v}`;
	const rest = body.slice(start + `## ${v}`.length);
	const end = rest.indexOf('\n## ');
	return (end === -1 ? rest : rest.slice(0, end)).trim() || `mason ${v}`;
}

run('git', ['tag', tag]);
run('git', ['push', 'origin', tag]);
console.log(`tagged ${tag}`);

execFileSync('gh', ['release', 'create', tag, '--title', `mason ${tag}`, '--notes', notesFor(version)], {
	cwd: root,
	stdio: 'inherit'
});
console.log(`released ${tag}`);
