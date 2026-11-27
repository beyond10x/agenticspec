/**
 * The model behind the animation, as reference tables read from the same data file: the stages
 * with marker, trust need and unit of work, the barriers with the decision that crosses them, and
 * the rungs with the architecture each one is.
 */
import type {ReactNode} from 'react';
import {ladder, sourceUrl, stageOf} from './data';
import './ladder.css';

export function StageTable(): ReactNode {
  return (
    <table className="ld-table">
      <thead>
        <tr>
          <th>stage</th>
          <th>marker (throughput)</th>
          <th>trust need</th>
          <th>unit of work</th>
        </tr>
      </thead>
      <tbody>
        {ladder.stages.map((s) => (
          <tr key={s.id}>
            <td>
              <b>{s.id}</b> {s.name}
            </td>
            <td>{s.marker}</td>
            <td className="ld-table__trust">{s.trust_need}</td>
            <td>{s.unit_of_work}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function BarrierTable(): ReactNode {
  return (
    <table className="ld-table">
      <thead>
        <tr>
          <th>barrier</th>
          <th>between</th>
          <th>the decision that crosses it</th>
        </tr>
      </thead>
      <tbody>
        {ladder.barriers.map((b) => (
          <tr key={b.id}>
            <td>
              <b>{b.id}</b> {b.name}
            </td>
            <td className="ld-table__nowrap">
              {b.between[0]} → {b.between[1]}
            </td>
            <td>{b.decision}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function RungTable(): ReactNode {
  return (
    <table className="ld-table">
      <thead>
        <tr>
          <th>rung</th>
          <th>stage</th>
          <th>architecture</th>
          <th>agent copies</th>
          <th>trust parts</th>
          {ladder.simulation && <th>tasks correct per day*</th>}
        </tr>
      </thead>
      <tbody>
        {ladder.rungs.map((r) => (
          <tr key={r.id}>
            <td>
              <a href={`#rung-${r.n}`}>
                {r.n}. {r.title}
              </a>
            </td>
            <td className="ld-table__nowrap">
              {r.stage} {stageOf(r.stage).name}
            </td>
            <td>
              <a href={sourceUrl(r.architecture)}>
                <code>{r.architecture.split('/').pop()}</code>
              </a>
            </td>
            <td className="ld-table__num">{r.agents_at_work}</td>
            <td className="ld-table__num">{r.trust_parts}</td>
            {ladder.simulation && <td className="ld-table__num">{Math.round(r.simulation?.tasks_correct ?? 0)}</td>}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function SimulationNote(): ReactNode {
  const s = ladder.simulation;
  if (!s) return null;
  return (
    <p className="ld-note">
      * Simulated with estimated rates, not measured. Each rung's architecture runs for {s.horizon} under the
      repository's behaviour profile, averaged over {s.seeds} seeds, and every agent copy at work is fed{' '}
      {s.tasks_per_hour_per_worker} tasks an hour. The simulator models agents, questions to you, reviews and
      schedules. It does not model a person reading diffs, sandboxes, grants, contracts or work that arrives on a
      channel, so those parts make the output safer without moving these numbers.
    </p>
  );
}
