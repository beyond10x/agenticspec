// The host components of the generated UI (`ess generate ui --target react`), copied into the
// generated app as `src/host-components.tsx` by `task ui:build`. One per host slot of
// `ui/sessionspec.ui.yaml`.
//
// SessionLanes: the session's events on one time axis, one lane per agent, sub-agents under
// their parent, a tick per event coloured by its kind.
import { useEffect, useState, type ReactNode } from "react";
import type { HostProps } from "./runtime/hosts";

type Row = { seq: number; at: string; agent: string; kind: string };

const TONE: Record<string, string> = {
  prompt: "#2563eb",
  model_turn: "#94a3b8",
  tool_call: "#0f766e",
  decision: "#d97706",
  tool_result: "#16a34a",
  compaction: "#a16207",
  exposure: "#dc2626",
  cache_bust: "#b91c1c",
};

function SessionLanes(props: HostProps<"sessionspec.view.SessionEvents">): ReactNode {
  const session = props.page.session_id ?? "";
  const [rows, setRows] = useState<Row[]>([]);
  useEffect(() => {
    let live = true;
    props.adapter
      .read({ view: "sessionspec.view.SessionEvents", params: { session_id: session } } as never)
      .then((result) => {
        if (live) {
          setRows(result.rows as unknown as Row[]);
        }
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [session]);
  if (rows.length === 0) {
    return <div>No event yet</div>;
  }
  const times = rows.map((r) => Date.parse(r.at));
  // A reduce, not `Math.min(...times)`: a session can hold more events than a call takes arguments.
  const from = times.reduce((a, b) => Math.min(a, b), Infinity);
  const to = Math.max(times.reduce((a, b) => Math.max(a, b), -Infinity), from + 1);
  const agents: string[] = [];
  for (const r of rows) {
    if (!agents.includes(r.agent)) {
      agents.push(r.agent);
    }
  }
  const width = 1000;
  const lane = 18;
  const x = (t: number) => ((t - from) / (to - from)) * (width - 160) + 150;
  return (
    <svg viewBox={`0 0 ${width} ${agents.length * lane + 10}`} width="100%" role="img" aria-label="Events by agent over time">
      {agents.map((a, i) => (
        <g key={a}>
          <text x={0} y={i * lane + 13} fontSize={11}>
            {a.length > 22 ? `${a.slice(0, 21)}…` : a}
          </text>
          <line x1={150} x2={width - 10} y1={i * lane + 9} y2={i * lane + 9} stroke="#e2e8f0" />
        </g>
      ))}
      {rows.map((r, k) =>
        r.kind === "turn_end" ? null : (
          <rect
            key={r.seq}
            x={x(times[k] ?? from)}
            y={agents.indexOf(r.agent) * lane + 3}
            width={1.5}
            height={12}
            fill={TONE[r.kind] ?? "#64748b"}
          />
        ),
      )}
    </svg>
  );
}

export const hosts = { SessionLanes };
