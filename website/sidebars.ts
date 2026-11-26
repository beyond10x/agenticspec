import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/** Document ids are the repository paths of the Markdown files, without `.md`. */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'sessionspec',
      collapsed: false,
      items: ['docs/design', 'docs/query-design', 'docs/interaction-records', 'ui/README'],
    },
    {
      type: 'category',
      label: 'Research',
      collapsed: false,
      items: ['research/2026-11-26-baseline'],
    },
  ],
};

export default sidebars;
