//! One integration-test binary (workspace rule "Tests stay fast"). Fixtures are synthetic.

use std::path::PathBuf;

use agenticspec::claude::{Cmd, classify, classify_with, lift};
use agenticspec::kpi::session_facts;
use agenticspec::replay::replay;
use agenticspec::time::{epoch_ms, rfc3339};
use agenticspec_types::cli as c;
use agenticspec_types::session as s;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/project/s1.jsonl")
}

fn report() -> c::KpiReport {
    let lifted = lift(&fixture()).unwrap();
    let replayed = replay(&lifted.steps);
    let from = epoch_ms("2026-11-25T00:00:00Z").unwrap();
    let to = epoch_ms("2026-11-26T00:00:00Z").unwrap();
    let mut acc = session_facts(&replayed.events, replayed.refused, from, to);
    acc.transcripts = 2;
    acc.report(from, to)
}

#[test]
fn time_round_trips() {
    let t = "2026-11-25T10:00:07.250Z";
    assert_eq!(rfc3339(epoch_ms(t).unwrap()), t);
    assert_eq!(epoch_ms("2026-11-25T11:00:07.250+01:00"), epoch_ms(t));
    assert_eq!(epoch_ms("not a time"), None);
}

#[test]
fn lift_replays_with_no_refusal() {
    let lifted = lift(&fixture()).unwrap();
    assert_eq!(lifted.agents, 2);
    let replayed = replay(&lifted.steps);
    assert_eq!(
        replayed.refused,
        0,
        "{:?}",
        replayed
            .log
            .iter()
            .filter(|l| l.refused)
            .map(|l| (l.command, &l.outcome))
            .collect::<Vec<_>>()
    );
    assert_eq!(replayed.log.len(), 23);
}

#[test]
fn sub_agent_brief_is_the_parent_known() {
    let lifted = lift(&fixture()).unwrap();
    let briefs: Vec<_> = lifted
        .steps
        .iter()
        .filter_map(|st| match &st.cmd {
            Cmd::SubmitPrompt(p) if p.agent_id.0 == "a1" => Some((p.author, p.attribution)),
            _ => None,
        })
        .collect();
    assert_eq!(
        briefs,
        vec![(s::PromptAuthor::ParentAgent, s::Attribution::Known)]
    );
}

#[test]
fn prompt_authors_follow_the_marks() {
    use s::Attribution::*;
    use s::PromptAuthor::*;
    assert_eq!(classify("plain question", false), (Human, Inferred));
    assert_eq!(
        classify("<task-notification>\n<task-id>x", false),
        (Harness, Known)
    );
    assert_eq!(
        classify(
            "<cross-session-message from=\"a\" from-name=\"loom\">hi",
            false
        ),
        (PeerSession, Known)
    );
    let delegates = ["operator".to_owned()];
    assert_eq!(
        classify_with(
            "<cross-session-message from=\"a\" from-name=\"operator\">hi",
            false,
            &delegates
        ),
        (Delegate, Inferred)
    );
    assert_eq!(
        classify_with(
            "<cross-session-message from=\"a\" from-name=\"operator\">hi",
            false,
            &[]
        ),
        (PeerSession, Known)
    );
    assert_eq!(
        classify("Typed by the session x on the owner's order: go on", false),
        (Delegate, Inferred)
    );
    assert_eq!(
        classify_with(
            "operator typed this on the owner's order",
            false,
            &delegates
        ),
        (Delegate, Inferred)
    );
    assert_eq!(
        classify("Your brief is the file /x", false),
        (PeerSession, Inferred)
    );
    assert_eq!(classify("Stop hook feedback: x", false), (Hook, Known));
}

#[test]
fn kpis_of_the_fixture() {
    let r = report();
    assert_eq!(r.sessions, 1);
    assert_eq!(r.sub_agents, 1);
    assert_eq!(r.refused_commands, 0);

    let opus = r
        .models
        .iter()
        .find(|m| m.model == "claude-opus-5-5")
        .unwrap();
    assert_eq!(
        (opus.scope, opus.messages, opus.output_tokens),
        (c::AgentScope::Main, 4, 73)
    );
    let sonnet = r
        .models
        .iter()
        .find(|m| m.model == "claude-sonnet-5-5")
        .unwrap();
    assert_eq!((sonnet.scope, sonnet.messages), (c::AgentScope::Sub, 1));

    // m4 comes 1 h 59 min after m3 on a 1-hour cache and reads nothing from it.
    assert_eq!(r.busts.len(), 1);
    assert_eq!(
        (
            r.busts[0].cause,
            r.busts[0].count,
            r.busts[0].tokens_written_again
        ),
        (c::BustCause::TtlExpired, 1, 30_400)
    );

    // Idle from 10:01:02 (m3 stops with end_turn) to 10:03:03 (human prompt) = 121 s waiting on
    // the human; 12:00:00 to 12:30:00 (delegate prompt) = 1800 s, under the 1 h split.
    assert_eq!(r.autonomy.waiting_on_human_seconds, 1921);
    assert_eq!(r.autonomy.away_seconds, 0);
    // At least one agent Working: 10:00:00 to 10:01:02 (main, with a1 inside it). 10:03:03 to
    // 12:00:00 is one gap over 10 minutes and does not count.
    assert_eq!(r.autonomy.working_seconds, 62);

    let decided = |d: s::Decider, x: s::Decision| {
        r.autonomy
            .decisions
            .iter()
            .find(|f| f.decider == d && f.decision == x)
            .map(|f| f.count)
    };
    assert_eq!(
        decided(s::Decider::PermissionMode, s::Decision::Accept),
        Some(1)
    );
    assert_eq!(decided(s::Decider::Hook, s::Decision::Refuse), Some(1));

    let exposure = |k: &str| {
        r.exposures
            .iter()
            .find(|e| e.kind == k)
            .map(|e| (e.verdict, e.channel, e.count))
    };
    assert_eq!(
        exposure("aws_access_key"),
        Some((s::Verdict::Ok, s::ExposureChannel::ToolOutput, 1))
    );
    assert_eq!(
        exposure("gitlab_token"),
        Some((s::Verdict::Unk, s::ExposureChannel::ToolOutput, 1))
    );
    assert_eq!(
        exposure("email"),
        Some((s::Verdict::Unk, s::ExposureChannel::Prompt, 1))
    );
}

mod sq {
    use agenticspec::sq;

    fn rows(text: &str) -> Vec<(Vec<String>, bool)> {
        let p = sq::compile(text).unwrap();
        let fb = sq::fact_base(&super::fixture()).unwrap();
        let a = sq::eval::run(&p, &fb);
        let mut out: Vec<(Vec<String>, bool)> = a.queries[0]
            .rows
            .iter()
            .map(|(v, s)| (v.iter().map(ToString::to_string).collect(), s.is_empty()))
            .collect();
        out.sort();
        out
    }

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| (*x).to_owned()).collect()
    }

    #[test]
    fn a_rule_whose_filter_binds_nothing_is_refused() {
        let e = sq::compile("r(X) :- X > 3.").unwrap_err();
        assert!(e.contains("can never run"), "{e}");
        assert!(
            sq::compile("?- nope{x: X}.")
                .unwrap_err()
                .contains("unknown relation")
        );
    }

    #[test]
    fn a_hook_refusal_joins_its_call() {
        let r = rows(
            "?- call{node: C, id: I, tool: T}, decided{call: I, decision: \"Refuse\", decider: D}.",
        );
        assert_eq!(r, vec![(v(&["s1#10", "t2", "Agent", "Hook"]), true)]);
    }

    #[test]
    fn an_inferred_author_makes_the_row_possible() {
        let r = rows("?- prompt{node: P, author: \"Human\"}.");
        assert_eq!(
            r,
            vec![(v(&["s1#19"]), true), (v(&["s1#2"]), false)],
            "the queued prompt is Known, the plain one Inferred"
        );
    }

    #[test]
    fn seq_takes_the_nearest_unless_asked_for_all() {
        let q = |pick: &str| {
            rows(&format!(
                "?- prompt{{node: P, agent: \"main\", class: \"Task\"}}, seq(P, C, scope: agent{pick}), call{{node: C, tool: T}}."
            ))
        };
        assert_eq!(q(""), vec![(v(&["s1#2", "s1#4", "Bash"]), true)]);
        assert_eq!(q(", pick: all").len(), 2);
        assert!(
            q(", within: 1s").is_empty(),
            "the call came 6 s after the prompt"
        );
    }

    #[test]
    fn a_wildcard_in_an_aggregate_is_counted() {
        let r = rows("?- session{id: S}, N = count{ call{node: _} }.");
        assert_eq!(r, vec![(v(&["s1", "2"]), true)]);
    }

    #[test]
    fn the_recorded_spawn_link_is_definite() {
        assert_eq!(rows("?- spawned(C, A)."), vec![(v(&["s1#10", "a1"]), true)]);
    }

    #[test]
    fn a_check_gives_a_verdict_per_session() {
        let src = "refused(C) :- decided{node: C, decision: \"Refuse\"}.\n\
                   check no-refusal { about \"x\" severity low expect none find refused }\n\
                   check some-refusal { about \"x\" severity low expect some find refused }";
        let p = sq::compile(src).unwrap();
        let fb = sq::fact_base(&super::fixture()).unwrap();
        let results = vec![sq::SessionAnswers {
            session_id: "s1".to_owned(),
            answers: sq::eval::run(&p, &fb),
        }];
        let r = sq::report(&p, &results);
        let figures: Vec<(i64, i64, i64)> = r.checks.iter().map(|c| (c.ok, c.gap, c.unk)).collect();
        assert_eq!(figures, vec![(0, 1, 0), (1, 0, 0)]);
    }
}

mod mining {
    use agenticspec::mine::{self, Level};

    fn corpus() -> Vec<mine::Mined> {
        mine::corpus(&[super::fixture()], Level::Tool)
    }

    #[test]
    fn a_turn_becomes_tokens_without_content() {
        let c = corpus();
        let toks: Vec<Vec<String>> = c[0]
            .turns
            .iter()
            .map(|t| t.tokens.iter().map(|x| x.token.clone()).collect())
            .collect();
        assert_eq!(
            toks,
            vec![vec!["Bash".to_owned(), "Agent/refused:Hook".to_owned()]]
        );
    }

    #[test]
    fn a_mined_pattern_compiles_to_a_query_that_finds_it() {
        let c = corpus();
        let r = mine::mine(&c, Level::Tool, 1, 1, 10);
        assert_eq!(r.patterns.len(), 1);
        let p = &r.patterns[0];
        assert_eq!(p.sessions, 1);
        let prog = agenticspec::sq::compile(&p.query).unwrap();
        let fb = agenticspec::sq::fact_base(&super::fixture()).unwrap();
        let a = agenticspec::sq::eval::run(&prog, &fb);
        assert_eq!(a.queries[0].rows.len(), 1, "{}", p.query);
    }

    #[test]
    fn a_procedure_types_its_steps() {
        let c = corpus();
        let toks = vec!["Bash".to_owned(), "Agent/refused:Hook".to_owned()];
        let p = mine::procedure(&c, Level::Tool, &toks).unwrap();
        let kinds: Vec<String> = p.steps.iter().map(|s| format!("{:?}", s.kind)).collect();
        assert_eq!(kinds, vec!["Program", "Child"]);
        assert_eq!(p.steps[1].after, vec!["s1".to_owned()]);
        assert_eq!(p.matches, 1);
    }
}

#[test]
fn a_message_is_priced_by_the_longest_model_prefix() {
    use agenticspec::pricing::cost_nanodollars;
    let u = s::Usage {
        uncached_input: 1000,
        cache_read: 10_000,
        cache_write_5m: 1000,
        cache_write_1h: 0,
        output: 100,
    };
    assert_eq!(cost_nanodollars("claude-opus-5-5", &u), Some(13_000_000));
    assert_eq!(cost_nanodollars("claude-opus-5", &u), Some(18_750_000));
    assert_eq!(cost_nanodollars("<synthetic>", &u), None);
}

mod blocks;
mod sim;
mod store;
