// The DOM events a page of the package sends instead of changing the host's URL or sending a
// command itself: the host acts on them, or not. Both bubble and cross a shadow root.

/** `agentic-navigate`: a link to a part page or to the overview of a level was followed. */
export interface NavigateDetail {
  architecture: string;
  part?: string;
  page: string;
  instance?: number;
}

/** `agentic-command`: an action was chosen; `input` holds what its row binds. */
export interface CommandDetail {
  architecture: string;
  part: string;
  command: string;
  action: string;
  input: Record<string, unknown>;
  instance?: number;
}

export function emitNavigate(from: Element | null, detail: NavigateDetail): void {
  from?.dispatchEvent(new CustomEvent('agentic-navigate', { detail, bubbles: true, composed: true, cancelable: true }));
}

export function emitCommand(from: Element | null, detail: CommandDetail): void {
  from?.dispatchEvent(new CustomEvent('agentic-command', { detail, bubbles: true, composed: true, cancelable: true }));
}
