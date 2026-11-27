/**
 * The doc page layout of the classic theme (3.10), with the desktop table of contents first in the
 * document order and drawn on the right by CSS `order`. A keyboard user who skips to the main
 * content reaches "On this page" before the links of the article, instead of after every one of
 * them (ux/site/findings/SUF-005). The docs-system's wrapper (a doc with no headings takes the
 * whole row) is kept.
 */
import type {ReactNode} from 'react';
import clsx from 'clsx';
import {useWindowSize} from '@docusaurus/theme-common';
import {useDoc} from '@docusaurus/plugin-content-docs/client';
import DocItemPaginator from '@theme/DocItem/Paginator';
import DocVersionBanner from '@theme/DocVersionBanner';
import DocVersionBadge from '@theme/DocVersionBadge';
import DocItemFooter from '@theme/DocItem/Footer';
import DocItemTOCMobile from '@theme/DocItem/TOC/Mobile';
import DocItemTOCDesktop from '@theme/DocItem/TOC/Desktop';
import DocItemContent from '@theme/DocItem/Content';
import DocBreadcrumbs from '@theme/DocBreadcrumbs';
import ContentVisibility from '@theme/ContentVisibility';
import styles from './styles.module.css';

interface Props {
  children: ReactNode;
}

export default function DocItemLayout({children}: Props): ReactNode {
  const {frontMatter, toc, metadata} = useDoc();
  const windowSize = useWindowSize();
  const hidden = !!frontMatter.hide_table_of_contents;
  const canRender = !hidden && toc.length > 0;
  const desktop = canRender && (windowSize === 'desktop' || windowSize === 'ssr');
  const layout = (
    <div className="row">
      {desktop && (
        <div className={clsx('col col--3', styles.tocCol)}>
          <DocItemTOCDesktop />
        </div>
      )}
      <div className={clsx('col', !hidden && styles.docItemCol)}>
        <ContentVisibility metadata={metadata} />
        <DocVersionBanner />
        <div className={styles.docItemContainer}>
          <article>
            <DocBreadcrumbs />
            <DocVersionBadge />
            {canRender && <DocItemTOCMobile />}
            <DocItemContent>{children}</DocItemContent>
            <DocItemFooter />
          </article>
          <DocItemPaginator />
        </div>
      </div>
    </div>
  );
  return hidden || toc.length > 0 ? layout : <div className="b10x-doc-item--no-toc">{layout}</div>;
}
