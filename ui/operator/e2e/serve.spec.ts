import { readFileSync } from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

// `agenticspec ui serve` of the workstation (global-setup.ts): opening its URL shows the
// operator UI over the default control plane on fixtures. The page sets the session cookie, the
// app reads the runtime interface at config.json's api_base, and the panels draw the fixtures'
// rows.
const base = () => process.env.SERVE_URL ?? '';
const fixtures = path.resolve(import.meta.dirname, '../../views/fixtures');
const rows = (view: string, key: string) =>
  (JSON.parse(readFileSync(path.join(fixtures, `${view}.json`), 'utf8')) as { rows: Record<string, string>[] }).rows.map((r) => r[key]);

test('ui serve shows the operator UI with rows from the fixtures', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  const config = await (await page.request.get(`${base()}/config.json`)).json();
  expect(config.api_base).toBe('/ingress/operator');
  await page.goto(`${base()}/#/part/gateway`);
  await expect(page.locator('h2.part-title')).toHaveText('gateway');
  const gateways = page.locator('[data-panel="gateway_gateways"] tr[data-row]');
  await expect(gateways.first()).toBeAttached();
  const keys = await gateways.evaluateAll((rs) => rs.map((r) => r.getAttribute('data-row')));
  for (const id of rows('agentic.gateway.Gateways', 'gateway_id')) expect(keys).toContain(id);
  // R5 allows every action, and R4 records the command the action sends.
  await page.goto(`${base()}/#/part/operator`);
  await page.getByRole('button', { name: 'Send a message' }).click();
  await expect(page.getByText('Send a message: done')).toBeVisible();
  const recorded = (await (await page.request.get(`${base()}/fixture/commands`)).json()) as { scope: string; command: string }[];
  expect(recorded).toContainEqual(expect.objectContaining({ scope: 'workstation/operator', command: 'agentic.channel.SendMessage' }));
});

// The control-plane UI beside it at /control/: the overview's tiles, then each page across parts
// renders from the same fixtures, a row action of Operations sends its command at the part's
// scope, and the Events page lists the level's channel as it plays.
test('ui serve shows the control-plane UI at /control/ with each page across parts', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  const config = await (await page.request.get(`${base()}/control/config.json`)).json();
  expect(config).toMatchObject({ mode: 'control-plane', api_base: '/ingress/operator' });
  const layout = await (await page.request.get(`${base()}/control/views.json`)).json();
  const pages = layout.control.pages as { page: string; label: string; kind: string }[];
  expect(pages.map((p) => p.label)).toEqual(['Inbox', 'Activity', 'Operations', 'Metrics', 'Architecture', 'Events']);

  await page.goto(`${base()}/control/`);
  await expect(page.locator('[data-panel="tiles"] [data-tile]').first()).toBeVisible();
  await expect(page.locator('[data-tile="cost_usd"]')).toBeVisible();

  for (const p of pages) {
    await page.goto(`${base()}/control/#/${p.page.split('.').join('/')}`);
    await expect(page.locator(`section.control-page[data-page="${p.page}"] > h2.page-title`)).toHaveText(p.label);
  }

  await page.goto(`${base()}/control/#/control/inbox`);
  await expect(page.locator('[data-view="agentic.question.OpenQuestions"][data-part="owner"] tr[data-row]').first()).toBeAttached();

  await page.goto(`${base()}/control/#/control/operations`);
  const gateways = page.locator('[data-view="agentic.gateway.Gateways"][data-part="gateway"]');
  await expect(gateways.locator('tr[data-row]').first()).toBeAttached();
  // DrainGateway moves a serving gateway; the fixture refuses it on one that is not.
  await gateways.locator('tr[data-row]', { hasText: 'Serving' }).first().getByRole('button', { name: 'Drain a gateway' }).click();
  await expect(page.getByText('Drain a gateway: done')).toBeVisible();
  const recorded = (await (await page.request.get(`${base()}/fixture/commands`)).json()) as { scope: string; command: string }[];
  expect(recorded).toContainEqual(expect.objectContaining({ scope: 'workstation/gateway', command: 'agentic.gateway.DrainGateway' }));

  await page.goto(`${base()}/control/#/control/metrics`);
  await expect(page.locator('[data-panel="metrics"] tr[data-row]')).toHaveCount(layout.control.metrics.length);

  await page.goto(`${base()}/control/#/control/activity`);
  await expect(page.locator('[data-panel="activity_gateway_calls"]')).toBeAttached();

  await page.goto(`${base()}/control/#/control/events`);
  await expect(page.locator('[data-panel="events"] tr[data-row]').first()).toBeAttached({ timeout: 15_000 });
});
