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
