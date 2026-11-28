// The command hooks of the studio: what the document cannot say yet (its `unmapped` entries).
import type { CommandHooks } from './generated/runtime';

export const hooks: CommandHooks = {
  // ess-ui/1 has no navigation on a command's outcome: an opened draft is shown by its page.
  'studio.draft.OpenDraft': (row, ctx) => {
    const id = row?.['draft_id'];
    if (typeof id === 'string' && id) ctx.navigate('draft', { draft_id: id });
  },
};
