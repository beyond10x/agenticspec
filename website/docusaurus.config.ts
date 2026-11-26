import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {withProductSite} from '@beyond10x/docs-system/product-site';

// The documentation pages are the user documentation in docs/, read in place from the repository
// root. Nothing is copied into website/. The internal notes in design/ and research/ are not
// published. The table below gives each file its route, its title and the header the docs-system
// theme draws over it (lede, status, source link); the files themselves stay as they are on
// GitHub. The landing page is src/pages/index.tsx, which draws product.json with the docs-system
// product template.

const siteDir = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(siteDir, '..');
const sourceBase = 'https://github.com/beyond10x/agentspec/blob/main/';

interface Page {
  slug: string;
  title: string;
  sidebar_label?: string;
  lede: string;
  status?: 'planned';
}

const pages: Record<string, Page> = {
  'docs/index.md': {
    slug: '/',
    title: 'What agentspec is',
    sidebar_label: 'Overview',
    lede: 'Numbers about your coding-agent sessions, computed on your machine from their transcripts.',
  },
  'docs/getting-started.md': {
    slug: '/getting-started',
    title: 'Getting started',
    lede: 'Install agentspec, fill the store and open the dashboard.',
  },
  'docs/dashboard.md': {
    slug: '/dashboard',
    title: 'Dashboard',
    lede: 'The live web UI, its data endpoints and the dashboard file.',
  },
  'docs/kpis.md': {
    slug: '/kpis',
    title: 'KPIs',
    lede: 'Each figure of the KPI report and how to read it.',
  },
  'docs/queries.md': {
    slug: '/queries',
    title: 'Queries',
    lede: 'Find the places in your sessions where something happened, with the sq query language.',
  },
  'docs/patterns.md': {
    slug: '/patterns',
    title: 'Patterns and procedures',
    lede: 'Find repeated runs of tool calls and export them as draft procedures.',
  },
  'docs/privacy.md': {
    slug: '/privacy',
    title: 'Privacy',
    lede: 'What the store holds, what it does not, and how secrets are handled.',
  },
  'docs/library.md': {
    slug: '/library',
    title: 'Records library',
    lede: 'Write session records from your own program, such as an LLM gateway.',
  },
};

const config: Config = {
  title: 'agentspec',
  tagline: 'Agent sessions as typed records you can measure and query.',

  future: {v4: true},
  url: 'https://beyond10x.github.io',
  baseUrl: '/agentspec/',
  organizationName: 'beyond10x',
  projectName: 'agentspec',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',

  markdown: {
    format: 'detect',
    hooks: {onBrokenMarkdownLinks: 'throw'},
    parseFrontMatter: async (params) => {
      const result = await params.defaultParseFrontMatter(params);
      const file = path.relative(repositoryRoot, params.filePath).split(path.sep).join('/');
      const page = pages[file];
      if (!page) return result;
      return {
        ...result,
        frontMatter: {
          ...result.frontMatter,
          ...page,
          source: file,
          source_url: sourceBase + file,
        },
      };
    },
  },
  i18n: {defaultLocale: 'en', locales: ['en']},

  presets: [
    [
      'classic',
      {
        docs: {
          path: '..',
          include: ['docs/*.md'],
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/agentspec/tree/main/',
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'agentspec',
      items: [
        {to: '/docs/getting-started', label: 'Getting started', position: 'left'},
        {to: '/docs/kpis', label: 'KPIs', position: 'left'},
        {to: '/docs/queries', label: 'Queries', position: 'left'},
        {to: '/docs/privacy', label: 'Privacy', position: 'left'},
        {href: 'https://github.com/beyond10x/agentspec', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Overview', to: '/docs/'},
            {label: 'Getting started', to: '/docs/getting-started'},
            {label: 'Dashboard', to: '/docs/dashboard'},
            {label: 'KPIs', to: '/docs/kpis'},
            {label: 'Queries', to: '/docs/queries'},
            {label: 'Patterns and procedures', to: '/docs/patterns'},
            {label: 'Privacy', to: '/docs/privacy'},
            {label: 'Records library', to: '/docs/library'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/agentspec'},
            {label: 'README', href: 'https://github.com/beyond10x/agentspec/blob/main/README.md'},
          ],
        },
      ],
      copyright: 'A beyond10x project · Apache-2.0 · built with Docusaurus and the docs-system product template.',
    },
  } satisfies Preset.ThemeConfig,
};

const site = withProductSite(config, {mark: 'S'});

// withProductSite adds its static directories as absolute paths, which the build would ship in its
// bundle; they are made relative to the site so no machine path reaches the built site.
export default {
  ...site,
  staticDirectories: site.staticDirectories?.map((directory) =>
    directory && path.isAbsolute(directory) ? path.relative(siteDir, directory) : directory,
  ),
} satisfies Config;
