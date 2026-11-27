import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';
import generated from './src/generated/sidebar.json';

type Items = Extract<SidebarsConfig[string], unknown[]>;

/**
 * Document ids are the repository paths of the Markdown files, without `.md`. The building blocks
 * and architectures are listed from src/generated/sidebar.json, which agenticspec-docs writes with
 * their pages.
 */
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
    {
      type: 'category',
      label: 'Building blocks',
      collapsed: false,
      link: {type: 'doc', id: 'docs/blocks/index'},
      items: generated.blocks as Items,
    },
    {
      type: 'category',
      label: 'Architectures',
      collapsed: true,
      link: {type: 'doc', id: 'docs/architectures/index'},
      items: generated.architectures,
    },
    'docs/kpis-catalogue',
  ],
};

export default sidebars;
