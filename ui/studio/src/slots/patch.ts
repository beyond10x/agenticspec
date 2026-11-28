// The tokens of an agenticspec-patch/1 patch for the patch editor: the verbs, variables, strings,
// numbers, punctuation and `%` comments of its rules.
import type { CodeToken } from '@kit';

export const VERBS = ['set', 'add', 'remove', 'connect', 'replicate', 'wrap', 'prepend', 'append', 'mark', 'unmark'];

const RULE =
  /(%[^\n]*)|("(?:[^"\\\n]|\\.)*"?)|(-?\b\d+(?:\.\d+)?\b)|(\b[a-z_][\w]*(?=\s*[({]))|(\b[A-Z_][\w]*\b)|(:-|[(){}[\].,:;!=<>])|(\s+|[^\s%"(){}[\].,:;!=<>]+?(?=[\s%"(){}[\].,:;!=<>]|$)|.)/gy;

/** Cuts `text` into tokens whose texts, joined, are `text` again. */
export function tokens(text: string): CodeToken[] {
  const out: CodeToken[] = [];
  RULE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while (RULE.lastIndex < text.length && (m = RULE.exec(text))) {
    const [all, comment, string, number, head, variable, punct] = m;
    let kind: CodeToken['kind'];
    if (comment) kind = 'comment';
    else if (string) kind = 'string';
    else if (number) kind = 'number';
    else if (head) kind = VERBS.includes(head) ? 'verb' : 'keyword';
    else if (variable) kind = 'var';
    else if (punct) kind = 'punct';
    const last = out[out.length - 1];
    if (!kind && last && !last.kind) last.text += all;
    else out.push(kind ? { text: all, kind } : { text: all });
  }
  return out;
}
