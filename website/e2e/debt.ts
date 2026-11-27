// The recorded debt of the docs-site gates: each test title (or, for the gates, a prefix of
// `<scenario> <viewport> <theme>`) that fails on the current site, with the finding of
// ux/site/findings that names its defect. Such a test is marked test.fail: the gate stays green
// and the debt stays visible; the test turns red the day the defect is gone, and the line goes
// with the fix. Empty since S-01's change set closed SUF-001 (the five architecture frames).

export const EXPECTED_FAIL: Record<string, string> = {};

export function expectedFail(title: string): string | undefined {
  return Object.entries(EXPECTED_FAIL).find(([k]) => title === k || title.startsWith(`${k} `))?.[1];
}
