// Renders static/favicon.svg into the raster sizes that platforms still
// demand (Safari's touch icon, Android/PWA, and a 32px legacy fallback).
// Run with: pnpm icons
import { chromium } from 'playwright-core';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { readFileSync } from 'node:fs';

const here = dirname(fileURLToPath(import.meta.url));
const svg = readFileSync(resolve(here, '../static/favicon.svg'), 'utf8');

/** apple-touch-icon must not be transparent and wants a little breathing room,
 *  so it gets the kiln field as a full bleed plate. */
const targets = [
	{ file: 'favicon-32.png', size: 32, pad: 0 },
	{ file: 'icon-192.png', size: 192, pad: 0 },
	{ file: 'icon-512.png', size: 512, pad: 0 },
	{ file: 'apple-touch-icon.png', size: 180, pad: 18 }
];

const browser = await chromium.launch({ channel: 'chrome', headless: true });

for (const { file, size, pad } of targets) {
	const page = await browser.newPage({ viewport: { width: size, height: size } });
	const inner = size - pad * 2;
	await page.setContent(
		`<!doctype html><style>
			*{margin:0;padding:0}
			body{width:${size}px;height:${size}px;background:${pad ? '#20120a' : 'transparent'};
			     display:grid;place-items:center}
			svg{width:${inner}px;height:${inner}px;display:block}
		 </style>${svg}`,
		{ waitUntil: 'load' }
	);
	await page.screenshot({
		path: resolve(here, '../static', file),
		omitBackground: pad === 0
	});
	await page.close();
	console.log(`wrote static/${file} (${size}px)`);
}

await browser.close();
