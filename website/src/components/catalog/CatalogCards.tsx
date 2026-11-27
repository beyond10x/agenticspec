import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import './catalog.css';

interface Move {
  kpi: string;
  from: string;
  to: string;
  contribution: string;
}

export interface Entry {
  name: string;
  to: string;
  purpose: string;
  experiment: string;
  variants: number;
  seeds: number;
  fitness: {from: string; to: string; gain: string} | null;
  gains: Move[];
  tradeoff: Move | null;
  stage: {id: string; name: string} | null;
}

function MoveLine({m, kind}: {m: Move; kind: 'gain' | 'tradeoff'}): ReactNode {
  return (
    <li className={`as-cat-move as-cat-move--${kind}`}>
      <code>{m.kpi}</code>
      <span className="as-cat-move__vals">
        {m.from} → {m.to}
      </span>
      <span className="as-cat-move__fit" title="change of the fitness">
        {m.contribution}
      </span>
    </li>
  );
}

/** The catalog index: one card per optimized architecture, written by agenticspec-docs. */
export default function CatalogCards({entries}: {entries: Entry[]}): ReactNode {
  return (
    <ul className="as-cat">
      {entries.map((e) => (
        <li key={e.name} className="as-cat-card">
          <div className="as-cat-card__head">
            <h3 className="as-cat-card__name">
              <Link to={e.to}>{e.name}</Link>
            </h3>
            {e.stage && (
              <Link className="as-cat-stage" to="/docs/ladder" title={`Agentic ladder, stage ${e.stage.id}`}>
                {e.stage.id} · {e.stage.name}
              </Link>
            )}
          </div>
          <p className="as-cat-card__purpose">{e.purpose}</p>
          {e.fitness ? (
            <p className="as-cat-fit">
              <span className="as-cat-fit__label">Fitness</span>
              <span className="as-cat-fit__vals">
                {e.fitness.from} → <b>{e.fitness.to}</b>
              </span>
              <span className="as-cat-fit__gain">{e.fitness.gain}</span>
            </p>
          ) : (
            <p className="as-cat-fit">No variant passed the gates.</p>
          )}
          {(e.gains.length > 0 || e.tradeoff) && (
            <div className="as-cat-moves">
              {e.gains.length > 0 && (
                <>
                  <p className="as-cat-moves__label">Gained most</p>
                  <ul>
                    {e.gains.map((m) => (
                      <MoveLine key={m.kpi} m={m} kind="gain" />
                    ))}
                  </ul>
                </>
              )}
              {e.tradeoff && (
                <>
                  <p className="as-cat-moves__label">Trade-off</p>
                  <ul>
                    <MoveLine m={e.tradeoff} kind="tradeoff" />
                  </ul>
                </>
              )}
            </div>
          )}
          <p className="as-cat-card__meta">
            {e.variants} variants × {e.seeds} seeds · {e.experiment}
          </p>
        </li>
      ))}
    </ul>
  );
}
