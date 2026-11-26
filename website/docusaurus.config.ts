import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {withProductSite} from '@beyond10x/docs-system/product-site';

// The documentation pages are the repository's own Markdown, read in place from the repository
// root: docs/, research/ and ui/README.md. Nothing is copied into website/. The table below gives
// each file its route, its title and the header the docs-system theme draws over it (lede, status,
// source link); the files themselves stay as they are on GitHub. The landing page is
// src/pages/index.tsx, which draws product.json with the docs-system product template.

const siteDir = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(siteDir, '..');
const sourceBase = 'https://github.com/beyond10x/sessionspec/blob/main/';

const releaseState =
  'No release yet: build the sessionspec binary from source. It reads Claude Code transcripts; other harnesses and the gateway wire are adapters in progress.';

interface Page {
  slug: string;
  title: string;
  sidebar_label?: string;
  lede: string;
  status?: 'planned';
}

const pages: Record<string, Page> = {
  'docs/design.md': {
    slug: '/',
    title: 'Design',
    lede: `The session IR, its actors and attribution, redaction and the KPIs. ${releaseState}`,
  },
  'docs/query-design.md': {
    slug: '/query-design',
    title: 'sq: queries and mining',
    lede: 'The sq query language over the session IR, pattern mining and the export of a mined procedure.',
  },
  'docs/interaction-records.md': {
    slug: '/interaction-records',
    title: 'Interaction records',
    lede: 'One record format for agent interaction, appended from transcripts and from an LLM gateway, stored in SQLite first.',
    status: 'planned',
  },
  'ui/README.md': {
    slug: '/ui',
    title: 'User interface',
    lede: 'The UI as an ess-ui/1 document over the views of sessionspec.view, its fixtures and its tests.',
  },
  'research/2026-11-26-baseline.md': {
    slug: '/research/baseline',
    title: 'Baseline: 72 hours of sessions',
    lede: 'Aggregate numbers over 72 hours of Claude Code sessions on one workstation, measured before the first KPI implementation.',
  },
};

const config: Config = {
  title: 'sessionspec',
  tagline: 'Agent sessions as typed records you can measure and query.',

  future: {v4: true},
  url: 'https://beyond10x.github.io',
  baseUrl: '/sessionspec/',
  organizationName: 'beyond10x',
  projectName: 'sessionspec',
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
          include: ['docs/*.md', 'research/*.md', 'ui/README.md'],
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/sessionspec/tree/main/',
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'sessionspec',
      items: [
        {to: '/docs/', label: 'Design', position: 'left', activeBaseRegex: '^/sessionspec/docs/?$'},
        {to: '/docs/query-design', label: 'Queries', position: 'left'},
        {to: '/docs/interaction-records', label: 'Records', position: 'left'},
        {to: '/docs/research/baseline', label: 'Research', position: 'left'},
        {href: 'https://github.com/beyond10x/sessionspec', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Design', to: '/docs/'},
            {label: 'sq: queries and mining', to: '/docs/query-design'},
            {label: 'Interaction records', to: '/docs/interaction-records'},
            {label: 'User interface', to: '/docs/ui'},
            {label: 'Baseline', to: '/docs/research/baseline'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/sessionspec'},
            {label: 'README', href: 'https://github.com/beyond10x/sessionspec/blob/main/README.md'},
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
