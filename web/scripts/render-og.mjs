// Renders scripts/og-template.html to static/og.png (1200x630, @2x).
// Run with: pnpm og
import { chromium } from 'playwright-core';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const template = resolve(here, 'og-template.html');
const out = resolve(here, '../static/og.png');

const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage({
	viewport: { width: 1200, height: 630 },
	deviceScaleFactor: 2
});
await page.goto(`file://${template}`, { waitUntil: 'networkidle' });
await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(400);
await page.screenshot({ path: out });
await browser.close();
console.log(`wrote ${out}`);
