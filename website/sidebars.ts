import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';
import generated from './src/generated/sidebar.json';

type Items = Extract<SidebarsConfig[string], unknown[]>;

/**
 * Document ids are the repository paths of the Markdown files, without `.md`. The building blocks,
 * packs, fields and architectures are listed from src/generated/sidebar.json, which
 * agenticspec-docs writes with their pages.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    'docs/index',
    'docs/getting-started',
    'docs/ladder',
    {
      type: 'category',
      label: 'Using agenticspec',
      collapsed: false,
      items: ['docs/dashboard', 'docs/kpis', 'docs/queries', 'docs/patterns'],
    },
    'docs/privacy',
    'docs/library',
    'docs/scenarios',
    'docs/contract',
    {
      type: 'category',
      label: 'Building blocks',
      collapsed: false,
      link: {type: 'doc', id: 'docs/blocks/index'},
      items: generated.blocks as Items,
    },
    {
      type: 'category',
      label: 'Packs',
      collapsed: true,
      link: {type: 'doc', id: 'docs/packs/index'},
      items: generated.packs as Items,
    },
    {
      type: 'category',
      label: 'Fields',
      collapsed: true,
      link: {type: 'doc', id: 'docs/fields/index'},
      items: generated.fields as Items,
    },
    {
      type: 'category',
      label: 'Architectures',
      collapsed: true,
      link: {type: 'doc', id: 'docs/architectures/index'},
      items: generated.architectures,
    },
    {
      type: 'category',
      label: 'Catalog',
      collapsed: true,
      link: {type: 'doc', id: 'docs/catalog/index'},
      items: generated.catalog,
    },
    'docs/kpis-catalogue',
  ],
};

export default sidebars;
