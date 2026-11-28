import { readFileSync } from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

// The operator UI `agenticspec ui build` wrote for the workstation, over the fixture adapter of
// the run (global-setup.ts): the app reads its data files and draws the layout's pages.
const base = () => process.env.OPERATOR_URL ?? '';
const fixtures = path.resolve(import.meta.dirname, '../../views/fixtures');
const agents = (JSON.parse(readFileSync(path.join(fixtures, 'agentic.agent.Agents.json'), 'utf8')) as { rows: { agent_id: string }[] }).rows;

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
});

test('the operator page renders its actions and its panels, and an action sends its command', async ({ page }) => {
  await page.goto(`${base()}/#/part/operator`);
  await expect(page.locator('h2.part-title')).toHaveText('operator');
  const send = page.getByRole('button', { name: 'Send a message' });
  await expect(send).toBeVisible();
  const rows = page.locator('[data-panel="agent_agents"] tr[data-row]');
  await expect(rows.first()).toBeAttached();
  const keys = await rows.evaluateAll((rs) => rs.map((r) => r.getAttribute('data-row')));
  for (const a of agents) expect(keys).toContain(a.agent_id);
  await send.click();
  await expect(page.getByText('Send a message: done')).toBeVisible();
  const seen = (await (await page.request.get(`${base()}/seen`)).json()) as { scope: string; command: string }[];
  expect(seen).toContainEqual({ scope: 'workstation/operator', command: 'agentic.channel.SendMessage' });
});

test('the overview lists the parts and the contract, and a link opens the part page', async ({ page }) => {
  await page.goto(`${base()}/`);
  await expect(page).toHaveTitle('workstation');
  await expect(page.locator('[data-panel="contract"] tr[data-row]').first()).toBeAttached();
  await page.getByRole('link', { name: 'Open operator' }).click();
  await expect(page).toHaveURL(/#\/part\/operator$/);
  await expect(page.locator('h2.part-title')).toHaveText('operator');
});

test('an action the action set withholds is not shown', async ({ page, context }) => {
  await context.addCookies([{ name: 'r5', value: 'deny', url: base() }]);
  await page.goto(`${base()}/#/part/operator`);
  await expect(page.locator('[data-panel="agent_agents"] tr[data-row]').first()).toBeAttached();
  await expect(page.getByRole('button', { name: 'Send a message' })).toHaveCount(0);
});
