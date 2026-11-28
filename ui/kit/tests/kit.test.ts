// Unit tests of ui/kit: the select's keyboard and roles, the formatting of values, the zoom
// arithmetic, and that every colour comes from a token.
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { nextTick } from 'vue';
import KitSelect from '../src/components/KitSelect.vue';
import KitRecord from '../src/components/KitRecord.vue';
import KitSwitch from '../src/components/KitSwitch.vue';
import KitPanel from '../src/components/KitPanel.vue';
import { formatValue, isDigest, shortDigest, toneOf } from '../src/format';
import { fitScale, initialView, zoomAt } from '../src/zoom';
import type { FieldDef } from '../src/types';

const options = [
  { value: 'a', label: 'alpha' },
  { value: 'b', label: 'beta' },
  { value: 'c', label: 'gamma' },
];

describe('KitSelect', () => {
  it('is a combobox that opens a listbox of options', async () => {
    const w = mount(KitSelect, { props: { label: 'Architecture', options, modelValue: 'b' }, attachTo: document.body });
    const button = w.get('[role="combobox"]');
    expect(button.attributes('aria-haspopup')).toBe('listbox');
    expect(button.attributes('aria-expanded')).toBe('false');
    expect(button.text()).toContain('beta');
    await button.trigger('click');
    expect(button.attributes('aria-expanded')).toBe('true');
    const list = w.get('[role="listbox"]');
    expect(button.attributes('aria-controls')).toBe(list.attributes('id'));
    const opts = w.findAll('[role="option"]');
    expect(opts).toHaveLength(3);
    expect(opts[1].attributes('aria-selected')).toBe('true');
    expect(button.attributes('aria-activedescendant')).toBe(opts[1].attributes('id'));
    const label = w.get(`#${button.attributes('aria-labelledby')}`);
    expect(label.text()).toBe('Architecture');
    w.unmount();
  });

  it('moves with the arrow keys and chooses with Enter', async () => {
    const w = mount(KitSelect, { props: { label: 'Pick', options, modelValue: null }, attachTo: document.body });
    const button = w.get('[role="combobox"]');
    await button.trigger('keydown', { key: 'ArrowDown' });
    expect(button.attributes('aria-expanded')).toBe('true');
    await button.trigger('keydown', { key: 'ArrowDown' });
    await button.trigger('keydown', { key: 'ArrowDown' });
    await button.trigger('keydown', { key: 'ArrowDown' });
    await button.trigger('keydown', { key: 'Enter' });
    expect(w.emitted('update:modelValue')?.[0]).toEqual(['c']);
    expect(button.attributes('aria-expanded')).toBe('false');
    w.unmount();
  });

  it('jumps by typed letters, Home and End, and closes on Escape without choosing', async () => {
    const w = mount(KitSelect, { props: { label: 'Pick', options, modelValue: 'a' }, attachTo: document.body });
    const button = w.get('[role="combobox"]');
    await button.trigger('keydown', { key: 'g' });
    expect(button.attributes('aria-expanded')).toBe('true');
    const opts = w.findAll('[role="option"]');
    expect(button.attributes('aria-activedescendant')).toBe(opts[2].attributes('id'));
    await button.trigger('keydown', { key: 'Home' });
    expect(button.attributes('aria-activedescendant')).toBe(opts[0].attributes('id'));
    await button.trigger('keydown', { key: 'End' });
    expect(button.attributes('aria-activedescendant')).toBe(opts[2].attributes('id'));
    await button.trigger('keydown', { key: 'Escape' });
    expect(button.attributes('aria-expanded')).toBe('false');
    expect(w.emitted('update:modelValue')).toBeUndefined();
    w.unmount();
  });

  it('shows its placeholder until a value is chosen, and chooses by click', async () => {
    const w = mount(KitSelect, { props: { label: 'Pick', options, placeholder: 'Choose one' }, attachTo: document.body });
    expect(w.get('[role="combobox"]').text()).toContain('Choose one');
    await w.get('[role="combobox"]').trigger('click');
    await w.findAll('[role="option"]')[0].trigger('click');
    expect(w.emitted('update:modelValue')?.[0]).toEqual(['a']);
    w.unmount();
  });
});

describe('values', () => {
  const badge: FieldDef = {
    field: 'passed',
    label: 'Check',
    as: 'badge',
    labels: { true: 'passes', false: 'refuses' },
    tones: { true: 'success', false: 'danger' },
  };

  it('shows labels, tones, numbers, lists and empty values', () => {
    expect(formatValue(badge, true)).toBe('passes');
    expect(toneOf(badge, false)).toBe('danger');
    expect(toneOf({ tone: 'info' }, 'x')).toBe('info');
    expect(formatValue({ as: 'number' }, 12000)).toBe('12,000');
    expect(formatValue({ as: 'percent' }, 0.256)).toBe('25.6 %');
    expect(formatValue({ as: 'list' }, ['a', 'b'])).toBe('a, b');
    expect(formatValue({ as: 'toggle' }, true)).toBe('yes');
    expect(formatValue({ as: 'text' }, null)).toBe('—');
    expect(formatValue({ as: 'timestamp' }, '2026-11-28T09:05:00Z')).toBe('2026-11-28 09:05 UTC');
  });

  it('shows a digest short and keeps the whole value', async () => {
    const digest = '8f14e45fceea167a5a36dedd4bea2543c4b5f0d6c2a9b1e3d7f6a0b5c4d3e2f1';
    expect(isDigest(digest)).toBe(true);
    expect(isDigest('workstation')).toBe(false);
    expect(shortDigest(digest)).toBe('8f14e45fceea…');
    const w = mount(KitRecord, {
      props: { fields: [{ field: 'digest', label: 'Digest', as: 'text' }, { field: 'rule', label: 'Rule', as: 'number' }], row: { digest, rule: null } },
    });
    const dd = w.get('[data-field="digest"]');
    expect(dd.text()).toBe('8f14e45fceea…');
    expect(dd.attributes('data-value')).toBe(digest);
    expect(dd.get('code').attributes('title')).toBe(digest);
    expect(w.find('[data-field="rule"]').exists()).toBe(false);
  });
});

describe('controls', () => {
  it('a switch is a role switch that flips', async () => {
    const w = mount(KitSwitch, { props: { label: 'Dry run', modelValue: false } });
    const sw = w.get('[role="switch"]');
    expect(sw.attributes('aria-checked')).toBe('false');
    expect(w.get('label').attributes('for')).toBe(sw.attributes('id'));
    await sw.trigger('click');
    expect(w.emitted('update:modelValue')?.[0]).toEqual([true]);
  });

  it('a panel shows the state of its read and collapses', async () => {
    const w = mount(KitPanel, {
      props: { name: 'parts', title: 'Parts', collapsible: true, collapsed: true, state: 'failed', error: 'NoSuchDraft' },
      slots: { default: '<p class="slotted">rows</p>' },
    });
    const toggle = w.get('button[aria-expanded]');
    expect(toggle.attributes('aria-expanded')).toBe('false');
    await toggle.trigger('click');
    await nextTick();
    expect(toggle.attributes('aria-expanded')).toBe('true');
    expect(w.get('[role="alert"]').text()).toContain('NoSuchDraft');
    expect(w.find('.slotted').exists()).toBe(false);
    await w.setProps({ state: 'ready' });
    expect(w.find('.slotted').exists()).toBe(true);
  });
});

describe('zoom', () => {
  const viewport = { width: 1000, height: 700 };

  it('opens a small graph fitted, never above 100 %', () => {
    const v = initialView(viewport, { width: 400, height: 300 }, 0.85, 24);
    expect(v.scale).toBe(1);
    expect(v.x).toBe(300);
    const w = initialView(viewport, { width: 1100, height: 600 }, 0.85, 24);
    expect(w.scale).toBeCloseTo(fitScale(viewport, { width: 1100, height: 600 }, 24));
    expect(w.scale).toBeGreaterThanOrEqual(0.85);
  });

  it('opens a large graph at the readable scale, from the top', () => {
    const v = initialView(viewport, { width: 3000, height: 2400 }, 0.85, 24);
    expect(v.scale).toBe(0.85);
    expect(v.y).toBe(24);
    expect(13.5 * v.scale).toBeGreaterThanOrEqual(11);
  });

  it('zooms about the pointer, within its limits', () => {
    const v = zoomAt({ scale: 1, x: 0, y: 0 }, 2, 100, 50, { min: 0.1, max: 4 });
    expect(v).toEqual({ scale: 2, x: -100, y: -50 });
    expect(zoomAt({ scale: 3, x: 0, y: 0 }, 10, 0, 0, { min: 0.1, max: 4 }).scale).toBe(4);
  });
});

describe('tokens', () => {
  const kit = path.resolve(import.meta.dirname ?? path.join(process.cwd(), 'kit/tests'), '..');
  const src = path.join(kit, 'src/components');

  it('no component writes a colour; each reads a token', () => {
    for (const f of readdirSync(src)) {
      const text = readFileSync(`${src}/${f}`, 'utf8');
      expect(text, f).not.toMatch(/#[0-9a-fA-F]{3,8}\b(?![\w-])/);
      expect(text, f).not.toMatch(/\b(rgb|hsl)a?\(/);
    }
  });

  it('the primary button draws with the accent token', () => {
    const button = readFileSync(`${src}/KitButton.vue`, 'utf8');
    expect(button).toMatch(/\.tone-primary \{\n\s+background: var\(--kit-accent\);/);
    const tokens = readFileSync(path.join(kit, 'src/tokens.css'), 'utf8');
    expect(tokens.match(/--kit-accent:/g)).toHaveLength(2);
  });
});
