/**
 * The stage rail: the seven stages as rungs of a ladder laid on its side, the barriers as
 * dashed gates between them. A stage jumps to its first rung, a barrier to its interstitial.
 */
import type {ReactNode} from 'react';
import clsx from 'clsx';
import {ladder, steps, type Step} from './data';

export default function StageRail({
  step,
  current,
  onJump,
}: {
  step: Step;
  current: string;
  onJump: (index: number) => void;
}): ReactNode {
  const order = ladder.stages.map((s) => s.id);
  const at = order.indexOf(current);
  const barrierAt = step.kind === 'barrier' ? step.barrier : -1;
  const items: ReactNode[] = [];
  ladder.stages.forEach((s, i) => {
    const k = ladder.barriers.findIndex((b) => b.between[1] === s.id);
    if (k >= 0) {
      const b = ladder.barriers[k];
      const target = steps.findIndex((x) => x.kind === 'barrier' && x.barrier === k);
      const crossed = at >= i && barrierAt !== k;
      items.push(
        <li
          key={b.id}
          className={clsx(
            'ld-rail__barrier',
            barrierAt === k && 'is-current',
            crossed && 'is-crossed',
          )}>
          <button
            type="button"
            onClick={() => onJump(target)}
            aria-current={barrierAt === k ? 'step' : undefined}>
            <span className="ld-rail__gate" aria-hidden="true" />
            <span className="ld-rail__barrier-id">{b.id}</span>
            <span className="ld-sr"> barrier: {b.name}</span>
          </button>
        </li>,
      );
    }
    const first = steps.findIndex((x) => x.kind === 'rung' && x.rung === s.rungs[0]);
    const isCurrent = s.id === current && barrierAt < 0;
    items.push(
      <li
        key={s.id}
        className={clsx('ld-rail__stage', isCurrent && 'is-current', i < at && 'is-past', i === at && 'is-reached')}>
        <button
          type="button"
          onClick={() => onJump(first)}
          aria-current={isCurrent ? 'step' : undefined}>
          <span className="ld-rail__dot" aria-hidden="true">
            {s.id.slice(1)}
          </span>
          <span className="ld-sr">Stage {s.id.slice(1)}: </span>
          <span className="ld-rail__name">{s.name}</span>
        </button>
      </li>,
    );
  });
  return (
    <nav className="ld-rail" aria-label="Stages of the ladder">
      <ol>{items}</ol>
    </nav>
  );
}
