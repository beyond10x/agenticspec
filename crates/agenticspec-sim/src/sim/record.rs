//! The port uses of one recorded run: each use of a port of the building blocks by an instance of
//! a part, with when it began and ended, the use it lies inside and its counterparty. Kept only
//! when a run records (`simulate(.., record: true)`); a run that does not record, every run of a
//! search, draws and counts the same, so recording changes no result.

use super::{Answerer, Sim, Target, Who};

/// The ports whose uses the simulator records: model calls (through a gateway), builds in a pool,
/// questions to a principal or a delegate, delivery on a direct channel, posts on a shared one and
/// the firings of triggers. A use of any other port is not in a recorded run.
pub const RECORDED: &[&str] = &[
    "channel.message",
    "channel.post",
    "gateway.call",
    "model.inference",
    "question.question",
    "resource.claim",
    "trigger.firing",
];

/// One use of a port by an instance of a part.
#[derive(Debug, Clone, PartialEq)]
pub struct PortUse {
    pub port: &'static str,
    pub part: String,
    /// Its number among the copies of its part; 0 for a part of one instance.
    pub instance: usize,
    pub begin: f64,
    pub end: f64,
    /// The use, by its place in the run's list of uses, this one lies inside.
    pub within: Option<usize>,
    /// The part on the other side of the use.
    pub to: Option<String>,
    /// The channel part the use is carried over.
    pub over: Option<String>,
}

impl PortUse {
    pub(super) fn new(
        port: &'static str,
        part: &str,
        instance: usize,
        begin: f64,
        end: f64,
    ) -> Self {
        PortUse {
            port,
            part: part.to_owned(),
            instance,
            begin,
            end,
            within: None,
            to: None,
            over: None,
        }
    }

    pub(super) fn to(mut self, part: &str) -> Self {
        self.to = Some(part.to_owned());
        self
    }

    pub(super) fn over(mut self, channel: Option<&str>) -> Self {
        self.over = channel.map(str::to_owned);
        self
    }

    pub(super) fn within(mut self, use_: Option<usize>) -> Self {
        self.within = use_;
        self
    }
}

impl Sim<'_> {
    /// Keeps a use when the run records and it began inside the horizon; returns its place.
    pub(super) fn port_use(&mut self, u: PortUse) -> Option<usize> {
        if !self.record || u.begin >= self.m.horizon {
            return None;
        }
        self.uses.push(u);
        Some(self.uses.len() - 1)
    }

    /// The uses of one model call of `part`'s instance from `begin` to `end`: the gateway's call
    /// when the route passes one, and the inference inside it.
    pub(super) fn model_call(
        &mut self,
        part: &str,
        instance: usize,
        route: (usize, Option<usize>),
        begin: f64,
        end: f64,
    ) {
        if !self.record {
            return;
        }
        let m = self.m;
        let (model, gateway) = route;
        let model = &m.models[model];
        let call = gateway.and_then(|g| {
            self.port_use(
                PortUse::new("gateway.call", &m.gateways[g].part, 0, begin, end).to(model),
            )
        });
        self.port_use(
            PortUse::new("model.inference", part, instance, begin, end)
                .to(model)
                .within(call),
        );
    }

    /// The uses of one question of instance `i`, asked at `asked` and resolved at `resolved`:
    /// the asker's, to the part it asks; and when a delegate forwarded it to a principal at
    /// `forwarded`, the delegate's question inside it, carried over the delegate's channel, with
    /// the delivery of the question on that channel.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_question(
        &mut self,
        i: usize,
        target: Target,
        answerer: Answerer,
        forwarded: Option<f64>,
        channel: Option<usize>,
        asked: f64,
        resolved: f64,
    ) {
        if !self.record {
            return;
        }
        let m = self.m;
        let Who::Worker(kind) = self.insts[i].who else {
            return;
        };
        let to = match target {
            Target::Principal(p) => &m.principals[p].part,
            Target::Delegate(d) => &m.delegates[d].part,
        };
        let asker = &m.workers[kind].exec.part;
        let copy = self.insts[i].copy;
        let q =
            self.port_use(PortUse::new("question.question", asker, copy, asked, resolved).to(to));
        if let (Target::Delegate(d), Answerer::Principal(p), Some(at)) =
            (target, answerer, forwarded)
        {
            let delegate = &m.delegates[d].part;
            let principal = &m.principals[p].part;
            let over = channel.map(|c| m.channels[c].as_str());
            let fwd = self.port_use(
                PortUse::new("question.question", delegate, 0, at, resolved)
                    .to(principal)
                    .over(over)
                    .within(q),
            );
            if let Some(c) = over {
                self.port_use(
                    PortUse::new("channel.message", delegate, 0, at, at)
                        .to(principal)
                        .over(Some(c))
                        .within(fwd),
                );
            }
        }
    }

    /// The delivery, at `at`, of principal `p`'s answer to the delegate that forwarded instance
    /// `i`'s question, over channel `c`.
    pub(super) fn record_answer(&mut self, i: usize, p: usize, c: usize, at: f64) {
        if !self.record {
            return;
        }
        let m = self.m;
        let Who::Worker(kind) = self.insts[i].who else {
            return;
        };
        let Some(Target::Delegate(d)) = m.workers[kind].question else {
            return;
        };
        self.port_use(
            PortUse::new("channel.message", &m.principals[p].part, 0, at, at)
                .to(&m.delegates[d].part)
                .over(Some(&m.channels[c])),
        );
    }

    /// The firing of trigger `tr` at `fired`, delivered to instance `i`, whose run starts at
    /// `started`.
    pub(super) fn record_firing(&mut self, tr: usize, i: usize, fired: f64, started: f64) {
        if !self.record {
            return;
        }
        let m = self.m;
        let Who::Worker(kind) = self.insts[i].who else {
            return;
        };
        self.port_use(
            PortUse::new("trigger.firing", &m.triggers[tr].part, 0, fired, started)
                .to(&m.workers[kind].exec.part),
        );
    }
}
