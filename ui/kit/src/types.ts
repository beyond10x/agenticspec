/** The tone of a badge, a record or a toast: how it reads, never a colour. */
export type Tone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

/** How a field's value is shown. */
export type FieldAs = 'text' | 'number' | 'badge' | 'toggle' | 'list' | 'timestamp' | 'date' | 'time' | 'percent';

/** One field a table, a list or a record shows. Generated pages hand these to the kit. */
export interface FieldDef {
  field: string;
  label: string;
  as: FieldAs;
  /** The label of each value, by the value as a string. */
  labels?: Record<string, string>;
  /** The tone of each value, by the value as a string. */
  tones?: Record<string, Tone>;
  /** The tone of every value when `tones` names none. */
  tone?: Tone;
}

/** A row as the kit reads it. */
export type KitRow = Record<string, unknown>;

/** A value a select or a switch holds. */
export type KitScalar = string | number | boolean | null;

/** One option of a select. */
export interface KitOption {
  value: KitScalar;
  label: string;
}

/** A toast the shell shows. */
export interface KitToast {
  id: number;
  tone: Tone;
  text: string;
}

/** One token of a code box: a run of text and its kind, which picks its colour. */
export interface CodeToken {
  text: string;
  kind?: 'verb' | 'keyword' | 'string' | 'number' | 'var' | 'punct' | 'comment';
}
