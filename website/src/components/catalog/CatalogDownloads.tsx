import type {ReactNode} from 'react';
import useBaseUrl from '@docusaurus/useBaseUrl';
import './catalog.css';

interface File {
  file: string;
  label: string;
  about: string;
}

/** The files of a catalog entry, each a real file under website/static/catalog/<name>/. */
export default function CatalogDownloads({dir, files}: {dir: string; files: File[]}): ReactNode {
  const base = useBaseUrl(dir);
  return (
    <ul className="as-cat-dl">
      {files.map((f) => (
        <li key={f.file}>
          <span className="as-cat-dl__label">{f.label}</span>
          <a className="as-cat-dl__file" href={`${base}${f.file}`} download={f.file}>
            {f.file}
          </a>
          <span className="as-cat-dl__about">{f.about}</span>
        </li>
      ))}
    </ul>
  );
}
