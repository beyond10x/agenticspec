import type {ReactNode} from 'react';
import {iconPath} from './families';

interface Props {
  name: string;
  family: string;
  layer?: number;
  layerName?: string;
  planned?: boolean;
  counts: Partial<Record<'entities' | 'commands' | 'events' | 'views' | 'refusals' | 'kpis', number>>;
}

const LABELS: [keyof Props['counts'], string, string][] = [
  ['entities', 'entity', 'entities'],
  ['commands', 'command', 'commands'],
  ['events', 'event', 'events'],
  ['views', 'view', 'views'],
  ['refusals', 'refusal', 'refusals'],
  ['kpis', 'KPI', 'KPIs'],
];

/** The block's place and size at a glance: family colour, layer, and how much its contract holds. */
export default function BlockHeader({name, family, layer, layerName, planned, counts}: Props): ReactNode {
  return (
    <div className={`as-head as-fam-${family}`}>
      <div className="as-head__id">
        <svg className="as-head__icon" viewBox="0 0 14 14" aria-hidden="true">
          <path d={iconPath(name)} />
        </svg>
        <div className="as-head__layer">
          {planned ? (
            <>
              <b>planned</b>
              <span>not specified yet</span>
            </>
          ) : (
            <>
              <b>layer {layer}</b>
              <span>{layerName}</span>
            </>
          )}
        </div>
      </div>
      <div className="as-head__stats">
        {LABELS.filter(([key]) => counts[key] !== undefined).map(([key, one, many]) => {
          const n = counts[key] ?? 0;
          return (
            <div key={key} className={`as-stat${n === 0 ? ' as-stat--zero' : ''}`}>
              <b>{n}</b>
              <span>{n === 1 ? one : many}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
