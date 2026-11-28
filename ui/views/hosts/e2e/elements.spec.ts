import { readFileSync } from 'node:fs';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';

// The custom elements in a plain page and in a React page, over the fixture source of the run.
const fixtures = path.resolve(import.meta.dirname, '../../fixtures');
const agents = (JSON.parse(readFileSync(path.join(fixtures, 'agentic.agent.Agents.json'), 'utf8')) as { rows: { agent_id: string }[] }).rows;

for (const host of ['plain', 'react'] as const) {
  test.describe(`the ${host} page`, () => {
    async function open(page: Page): Promise<void> {
      await page.setViewportSize({ width: 1366, height: 768 });
      await page.goto(`${process.env.HOSTS_URL ?? ''}/hosts/${host}.html`);
      await expect(page.locator('agentic-part-panel [data-panel="agent_agents"] tr[data-row]').first()).toBeAttached();
    }

    test('shows the part page panels with the rows of the fixture', async ({ page }) => {
      await open(page);
      const keys = await page
        .locator('agentic-part-panel [data-panel="agent_agents"] tr[data-row]')
        .evaluateAll((rs) => rs.map((r) => r.getAttribute('data-row')));
      for (const a of agents) expect(keys).toContain(a.agent_id);
      await expect(page.locator('agentic-part-panel h2.part-title')).toHaveText('operator');
    });

    test('draws with the accent the host sets on an ancestor', async ({ page }) => {
      await open(page);
      const accent = await page
        .locator('agentic-overview a.open')
        .first()
        .evaluate((a) => getComputedStyle(a).color);
      expect(accent).toBe('rgb(200, 30, 120)');
    });

    test('a link and an action send their events and leave the URL alone', async ({ page }) => {
      await open(page);
      const before = page.url();
      await page.locator('agentic-overview a.open', { hasText: 'Open operator' }).click();
      await expect(page.locator('#last')).toContainText('agentic-navigate');
      await expect(page.locator('#last')).toContainText('"part":"operator"');
      expect(page.url()).toBe(before);
      await page.locator('agentic-part-panel').getByRole('button', { name: 'Send a message' }).click();
      await expect(page.locator('#last')).toContainText('agentic-command');
      await expect(page.locator('#last')).toContainText('"command":"agentic.channel.SendMessage"');
      expect(page.url()).toBe(before);
    });
  });
}
