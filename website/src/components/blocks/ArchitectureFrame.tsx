import {useCallback, useEffect, useRef, useState, type ReactNode} from 'react';
import useBaseUrl from '@docusaurus/useBaseUrl';
import {useColorMode} from '@docusaurus/theme-common';

interface Props {
  name: string;
  src: string;
  caption?: string;
}

/**
 * The rendered page of an architecture, embedded at its own height in the site's colour mode,
 * with a link that opens it on its own.
 */
export default function ArchitectureFrame({name, src, caption}: Props): ReactNode {
  const {colorMode} = useColorMode();
  const base = useBaseUrl(src);
  const url = `${base}?theme=${colorMode}`;
  const frame = useRef<HTMLIFrameElement>(null);
  const [height, setHeight] = useState(1100);

  const measure = useCallback(() => {
    const doc = frame.current?.contentDocument;
    if (!doc?.documentElement) return;
    const h = Math.max(doc.documentElement.scrollHeight, doc.body?.scrollHeight ?? 0);
    if (h > 0) setHeight(h + 2);
  }, []);

  useEffect(() => {
    const el = frame.current;
    if (!el) return undefined;
    let observer: ResizeObserver | undefined;
    const onLoad = () => {
      measure();
      const body = el.contentDocument?.body;
      if (body && typeof ResizeObserver !== 'undefined') {
        observer?.disconnect();
        observer = new ResizeObserver(measure);
        observer.observe(body);
      }
    };
    el.addEventListener('load', onLoad);
    if (el.contentDocument?.readyState === 'complete') onLoad();
    return () => {
      el.removeEventListener('load', onLoad);
      observer?.disconnect();
    };
  }, [measure, url]);

  return (
    <figure className="as-arch">
      <div className="as-arch__bar">
        <span>{caption}</span>
        <a href={url} target="_blank" rel="noopener">
          Open full screen ↗
        </a>
      </div>
      <iframe ref={frame} src={url} title={`The ${name} architecture, rendered`} style={{height}} />
    </figure>
  );
}
