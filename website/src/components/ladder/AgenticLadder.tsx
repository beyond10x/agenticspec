/**
 * The agentic ladder, animated: one owner's architecture grows rung by rung from one sandboxed
 * agent session to a factory of teams. A stage rail on top, the architecture canvas, the controls
 * (play, pause, previous, next, a scrubber, the arrow keys and space) and a panel with the stage,
 * its trust need and what the rung changed. Each rung has a deep link, #rung-3; each barrier,
 * #barrier-1. With prefers-reduced-motion the steps change without motion.
 */
import {useCallback, useEffect, useRef, useState, type KeyboardEvent, type ReactNode} from 'react';
import clsx from 'clsx';
import Canvas, {NARROW, WIDE} from './Canvas';
import StageRail from './StageRail';
import {hashOf, ladder, rungOf, sourceUrl, stageOf, stepOfHash, steps, type Rung} from './data';
import './ladder.css';

const RUNG_MS = 4200;
const BARRIER_MS = 5600;

function useReducedMotion(): boolean {
  const [reduced, setReduced] = useState(false);
  useEffect(() => {
    const q = window.matchMedia('(prefers-reduced-motion: reduce)');
    const on = () => setReduced(q.matches);
    on();
    q.addEventListener('change', on);
    return () => q.removeEventListener('change', on);
  }, []);
  return reduced;
}

function Icon({name}: {name: 'prev' | 'next' | 'play' | 'pause' | 'replay'}): ReactNode {
  const d = {
    prev: 'M10 3L5 8l5 5',
    next: 'M6 3l5 5-5 5',
    play: 'M5 3l8 5-8 5z',
    pause: 'M5 3v10M11 3v10',
    replay: 'M3.5 8a4.5 4.5 0 1 0 1.4-3.3M3.5 2.5v3h3',
  }[name];
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true" className={`ld-btn__icon ld-btn__icon--${name}`}>
      <path d={d} />
    </svg>
  );
}

function Shield(): ReactNode {
  return (
    <svg viewBox="-7 -7 14 14" className="ld-shield-inline" aria-hidden="true">
      <path d="M0 -6.2l5 2v3.4c0 2.9-2.1 4.6-5 5.4-2.9-.8-5-2.5-5-5.4v-3.4z" />
    </svg>
  );
}

const fmt = new Intl.NumberFormat('en', {maximumFractionDigits: 0});

function Stats({rung}: {rung: Rung}): ReactNode {
  const sim = rung.simulation;
  const max = Math.max(...ladder.rungs.map((r) => r.simulation?.tasks_correct ?? 0), 1);
  return (
    <div className="ld-stats">
      <div className="ld-stat">
        <span className="ld-stat__value">{rung.agents_at_work}</span>
        <span className="ld-stat__name">agent copies at work</span>
      </div>
      <div className="ld-stat ld-stat--trust">
        <span className="ld-stat__value">
          <Shield />
          {rung.trust_parts}
        </span>
        <span className="ld-stat__name">trust parts in force</span>
      </div>
      {sim && (
        <>
          <div className="ld-stat ld-stat--wide">
            <span className="ld-stat__value">{fmt.format(sim.tasks_correct)}</span>
            <span className="ld-stat__name">tasks correct per day*</span>
            <svg
              className="ld-spark"
              preserveAspectRatio="none"
              viewBox={`0 0 ${ladder.rungs.length * 12} 30`}
              role="img"
              aria-label={`Tasks correct per day, simulated, rung 1 to ${ladder.rungs.length}: ${ladder.rungs
                .map((r) => fmt.format(r.simulation?.tasks_correct ?? 0))
                .join(', ')}`}>
              {ladder.rungs.map((r, i) => {
                const v = r.simulation?.tasks_correct ?? 0;
                const h = Math.max(1.5, (v / max) * 28);
                return (
                  <rect
                    key={r.id}
                    className={clsx('ld-spark__bar', r.n === rung.n && 'is-current', r.n > rung.n && 'is-later')}
                    x={i * 12 + 1}
                    y={30 - h}
                    width={9}
                    height={h}
                    rx={1.5}
                  />
                );
              })}
            </svg>
          </div>
          <div className="ld-stat">
            <span className="ld-stat__value">{Math.round(sim.owner_wait_share * 100)} %</span>
            <span className="ld-stat__name">of agent time waiting on you*</span>
          </div>
          <div className="ld-stat">
            <span className="ld-stat__value">${fmt.format(sim.cost_usd)}</span>
            <span className="ld-stat__name">cost per day*</span>
          </div>
        </>
      )}
    </div>
  );
}

function Added({rung}: {rung: Rung}): ReactNode {
  const parts = rung.parts.filter((p) => rung.added_parts.includes(p.id) || rung.trust_added.includes(p.id));
  const changed = rung.parts.filter((p) => rung.changed_parts.includes(p.id) && !rung.added_parts.includes(p.id));
  if (parts.length === 0 && changed.length === 0 && rung.removed_parts.length === 0) return null;
  return (
    <div className="ld-added">
      <h3 className="ld-label">At this rung</h3>
      <ul>
        {parts.map((p) => (
          <li key={p.id} className={clsx('ld-chip', `as-fam-${p.family}`, p.trust && 'ld-chip--trust')}>
            <span className="ld-chip__sign" aria-hidden="true">+</span>
            <code>{p.id}</code>
            {p.count > 1 && <span className="ld-chip__count">×{p.count}</span>}
            {p.trust ? (
              <span className="ld-chip__trust">
                <Shield />
                <span className="ld-sr">trust part: </span>
                {p.trust}
              </span>
            ) : (
              <span className="ld-chip__label">{p.label}</span>
            )}
          </li>
        ))}
        {changed.map((p) => (
          <li key={p.id} className={clsx('ld-chip', 'ld-chip--changed', `as-fam-${p.family}`, p.trust && 'ld-chip--trust')}>
            <span className="ld-chip__sign" aria-hidden="true">~</span>
            <code>{p.id}</code>
            {p.count > 1 && <span className="ld-chip__count">×{p.count}</span>}
            <span className="ld-chip__label">{p.trust ?? p.label}</span>
          </li>
        ))}
        {rung.removed_parts.map((id) => (
          <li key={id} className="ld-chip ld-chip--removed">
            <span className="ld-chip__sign" aria-hidden="true">−</span>
            <code>{id}</code>
            <span className="ld-chip__label">removed</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default function AgenticLadder(): ReactNode {
  const [state, setState] = useState<{index: number; prev: number | undefined}>({index: 0, prev: undefined});
  const [playing, setPlaying] = useState(false);
  const [narrow, setNarrow] = useState(false);
  const reduced = useReducedMotion();
  const frame = useRef<HTMLDivElement>(null);

  const step = steps[state.index];
  const rungIndex = rungOf(step);
  const rung = ladder.rungs[rungIndex];
  const previous = state.prev === undefined ? undefined : ladder.rungs[state.prev];
  const stage = stageOf(rung.stage);
  const barrier = step.kind === 'barrier' ? ladder.barriers[step.barrier] : undefined;
  const farStage = barrier ? stageOf(barrier.between[1]) : undefined;
  const nearStage = barrier ? stageOf(barrier.between[0]) : undefined;
  const last = steps.length - 1;

  const go = useCallback((i: number, writeHash = true) => {
    const next = Math.max(0, Math.min(last, i));
    setState((s) => (s.index === next ? s : {index: next, prev: rungOf(steps[s.index])}));
    if (writeHash && typeof window !== 'undefined') {
      window.history.replaceState(null, '', `#${hashOf(steps[next])}`);
    }
  }, [last]);

  useEffect(() => {
    const fromHash = () => {
      const i = stepOfHash(window.location.hash);
      if (i !== null) go(i, false);
    };
    fromHash();
    window.addEventListener('hashchange', fromHash);
    return () => window.removeEventListener('hashchange', fromHash);
  }, [go]);

  useEffect(() => {
    const el = frame.current;
    if (!el) return undefined;
    const ro = new ResizeObserver((entries) => {
      for (const e of entries) setNarrow(e.contentRect.width < 640);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    if (!playing) return undefined;
    if (state.index >= last) {
      setPlaying(false);
      return undefined;
    }
    const t = window.setTimeout(() => go(state.index + 1), step.kind === 'barrier' ? BARRIER_MS : RUNG_MS);
    return () => window.clearTimeout(t);
  }, [playing, state.index, step.kind, last, go]);

  const toggle = () => {
    if (!playing && state.index >= last) go(0);
    setPlaying((p) => !p);
  };

  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    const target = e.target as HTMLElement;
    const tag = target.tagName;
    if (tag === 'INPUT' && e.key.startsWith('Arrow')) return;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      setPlaying(false);
      go(state.index + 1);
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      setPlaying(false);
      go(state.index - 1);
    } else if (e.key === 'Home') {
      e.preventDefault();
      go(0);
    } else if (e.key === 'End') {
      e.preventDefault();
      go(last);
    } else if (e.key === ' ' && tag !== 'BUTTON' && tag !== 'INPUT' && tag !== 'A') {
      e.preventDefault();
      toggle();
    }
  };

  const where = barrier
    ? `Barrier ${barrier.id.slice(1)} of ${ladder.barriers.length}: ${barrier.name}`
    : `Rung ${rung.n} of ${ladder.rungs.length}: ${rung.title}. Stage ${stage.id}, ${stage.name}.`;
  const canvasTitle = `Architecture at rung ${rung.n}, ${rung.title}: ${rung.parts.length} parts, of which ${rung.trust_parts} are trust parts. Parts: ${rung.parts
    .map((p) => (p.count > 1 ? `${p.id} (${p.count} copies)` : p.id))
    .join(', ')}.`;

  return (
    <div
      className={clsx('ld', reduced && 'ld--still', narrow && 'ld--narrow')}
      role="region"
      aria-label="The agentic ladder, animated. Arrow keys step through the rungs; space plays or pauses."
      tabIndex={0}
      onKeyDown={onKey}>
      <StageRail step={step} current={rung.stage} onJump={(i) => {
        setPlaying(false);
        go(i);
      }} />

      <div className="ld-stage" ref={frame}>
        <div className="ld-stage__head">
          <p className="ld-kicker">
            {barrier ? (
              <>
                <span className="ld-kicker__barrier">Barrier {barrier.id.slice(1)}</span> between {nearStage?.id}{' '}
                {nearStage?.name} and {farStage?.id} {farStage?.name}
              </>
            ) : (
              <>
                <span className="ld-kicker__stage">
                  {stage.id} · {stage.name}
                </span>{' '}
                <span className="ld-kicker__rung">
                  rung {rung.n} of {ladder.rungs.length}
                </span>
              </>
            )}
          </p>
          <h2 className="ld-title">{barrier ? barrier.name : rung.title}</h2>
        </div>
        <div className="ld-frame">
          <Canvas
            g={narrow ? NARROW : WIDE}
            rung={rung}
            previous={previous}
            dim={!!barrier}
            title={canvasTitle}
          />
          {barrier && (
            <div className="ld-barrier" key={barrier.id}>
              <div className="ld-barrier__gate" aria-hidden="true" />
              <div className="ld-barrier__card">
                <p className="ld-barrier__kicker">
                  The decision that crosses barrier {barrier.id.slice(1)}
                </p>
                <blockquote className="ld-barrier__decision">{barrier.decision}</blockquote>
                <p className="ld-barrier__note">
                  A barrier is a change of mental model, not of tooling. It is crossed only by what you
                  do on the far side.
                </p>
              </div>
            </div>
          )}
          {!barrier && rung.risk && (
            <p className="ld-risk" role="note">
              <strong>Risk.</strong> {rung.risk}
            </p>
          )}
        </div>

        <div className="ld-controls">
          <button
            type="button"
            className="ld-btn"
            onClick={() => {
              setPlaying(false);
              go(state.index - 1);
            }}
            disabled={state.index === 0}
            aria-label="Previous step">
            <Icon name="prev" />
          </button>
          <button
            type="button"
            className="ld-btn ld-btn--play"
            onClick={toggle}
            aria-label={playing ? 'Pause' : state.index >= last ? 'Play again from the first rung' : 'Play'}>
            <Icon name={playing ? 'pause' : state.index >= last ? 'replay' : 'play'} />
            <span className="ld-btn__text">{playing ? 'Pause' : state.index >= last ? 'Replay' : 'Play'}</span>
          </button>
          <button
            type="button"
            className="ld-btn"
            onClick={() => {
              setPlaying(false);
              go(state.index + 1);
            }}
            disabled={state.index === last}
            aria-label="Next step">
            <Icon name="next" />
          </button>
          <input
            className="ld-scrub"
            type="range"
            min={0}
            max={last}
            step={1}
            value={state.index}
            onChange={(e) => {
              setPlaying(false);
              go(Number(e.target.value));
            }}
            aria-label="Step"
            aria-valuetext={where}
            style={{['--ld-progress' as string]: `${(state.index / last) * 100}%`}}
          />
          <span className="ld-step" aria-hidden="true">
            {barrier ? barrier.id : `${rung.n}/${ladder.rungs.length}`}
          </span>
        </div>
        <p className="ld-sr" aria-live="polite">
          {where}
        </p>
      </div>

      <div className="ld-panel">
        {barrier && farStage ? (
          <>
            <div className="ld-panel__main">
              <h3 className="ld-label">On the far side: {farStage.id} · {farStage.name}</h3>
              <p className="ld-changed">{farStage.marker}</p>
            </div>
            <dl className="ld-facts">
              <div className="ld-fact ld-fact--trust">
                <dt>
                  <Shield /> Trust need
                </dt>
                <dd>{farStage.trust_need}</dd>
              </div>
              <div className="ld-fact">
                <dt>Unit of work</dt>
                <dd>{farStage.unit_of_work}</dd>
              </div>
            </dl>
          </>
        ) : (
          <>
            <div className="ld-panel__main">
              <h3 className="ld-label">What changed</h3>
              <p className="ld-changed">{rung.changed}</p>
              <Added rung={rung} />
              <p className="ld-source">
                This rung is the architecture document{' '}
                <a href={sourceUrl(rung.architecture)}>
                  <code>{rung.architecture.split('/').pop()}</code>
                </a>
                , accepted by <code>agenticspec sim check</code>.
              </p>
            </div>
            <dl className="ld-facts">
              <div className="ld-fact">
                <dt>Marker of {stage.id}</dt>
                <dd>{stage.marker}</dd>
              </div>
              <div className="ld-fact ld-fact--trust">
                <dt>
                  <Shield /> Trust need
                </dt>
                <dd>{stage.trust_need}</dd>
              </div>
              <div className="ld-fact">
                <dt>Unit of work</dt>
                <dd>{stage.unit_of_work}</dd>
              </div>
            </dl>
          </>
        )}
        <Stats rung={rung} />
      </div>
    </div>
  );
}
