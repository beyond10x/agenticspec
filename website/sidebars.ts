import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/** Document ids are the repository paths of the Markdown files, without `.md`. */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    'docs/index',
    'docs/getting-started',
    {
      type: 'category',
      label: 'Using agenticspec',
      collapsed: false,
      items: ['docs/dashboard', 'docs/kpis', 'docs/queries', 'docs/patterns'],
    },
    'docs/privacy',
    'docs/library',
  ],
};

export default sidebars;
