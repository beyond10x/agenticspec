/**
 * The MDX vocabulary of this site: the docs-system's, plus the components the generated pages of
 * the building blocks, the architectures and the KPI catalogue use without an import.
 */
import MDXComponents from '@theme-original/MDXComponents';
import ArchitectureFrame from '@site/src/components/blocks/ArchitectureFrame';
import BlockHeader from '@site/src/components/blocks/BlockHeader';
import BlockMap from '@site/src/components/blocks/BlockMap';
import KpiCatalogue from '@site/src/components/blocks/KpiCatalogue';
import Lifecycle from '@site/src/components/blocks/Lifecycle';
import '@site/src/components/blocks/blocks.css';

export default {
  ...MDXComponents,
  ArchitectureFrame,
  BlockHeader,
  BlockMap,
  KpiCatalogue,
  Lifecycle,
};
