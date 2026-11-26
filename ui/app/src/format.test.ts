import { describe, expect, it } from 'vitest';
import { ago, compact, duration, modelName, money, percent, projectName, shortDuration } from './format.ts';

describe('format', () => {
  it('reads money at the precision a reader needs', () => {
    expect(money(0)).toBe('$0');
    expect(money(0.004)).toBe('<$0.01');
    expect(money(23.617)).toBe('$23.62');
    expect(money(912.4)).toBe('$912');
    expect(money(39650.1)).toBe('$39.7K');
    expect(money(undefined)).toBe('–');
  });

  it('compacts large counts', () => {
    expect(compact(950)).toBe('950');
    expect(compact(345_513)).toBe('346K');
    expect(compact(12_900)).toBe('12.9K');
    expect(compact(436_000_000)).toBe('436M');
  });

  it('says durations in the largest two units', () => {
    expect(duration(45)).toBe('45 s');
    expect(duration(792)).toBe('13 min');
    expect(duration(3 * 3600 + 13 * 60)).toBe('3 h 13 min');
    expect(duration(3600 * 49)).toBe('2 d 1 h');
    expect(shortDuration(2 * 3600_000 + 13 * 60_000)).toBe('2h 13m');
  });

  it('says how long ago', () => {
    const now = Date.parse('2026-11-26T12:00:00Z');
    expect(ago(now - 10_000, now)).toBe('just now');
    expect(ago(now - 12 * 60_000, now)).toBe('12 min ago');
    expect(ago(now - 3 * 3600_000, now)).toBe('3 h ago');
    expect(ago(now - 30 * 3600_000, now)).toBe('yesterday');
  });

  it('shares and names', () => {
    expect(percent(0.9684)).toBe('97%');
    expect(percent(0.004)).toBe('<1%');
    expect(modelName('claude-opus-5-5')).toBe('opus 5.5');
    expect(modelName('model-demo-large')).toBe('model-demo-large');
    expect(projectName('~/src/ess')).toBe('src/ess');
    expect(projectName('alpha')).toBe('alpha');
  });
});
