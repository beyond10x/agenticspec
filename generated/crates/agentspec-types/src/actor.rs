// generated from agentspec v1
// contract digest 0c0727aa4de388310699c24fed8ce0f18713c396101cd870029ee370ff1fdebf
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `agentspec.session.AgentLoop`.
    AgentLoop,
    /// `agentspec.session.Delegate`.
    Delegate,
    /// `agentspec.session.HarnessRuntime`.
    HarnessRuntime,
    /// `agentspec.session.Human`.
    Human,
    /// `agentspec.session.PeerSession`.
    PeerSession,
    /// `agentspec.session.PermissionAuthority`.
    PermissionAuthority,
    /// `agentspec.session.Redactor`.
    Redactor,
    /// `agentspec.session.ToolProvider`.
    ToolProvider,
    /// `agentspec.view.Viewer`.
    Viewer,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::AgentLoop,
        Actor::Delegate,
        Actor::HarnessRuntime,
        Actor::Human,
        Actor::PeerSession,
        Actor::PermissionAuthority,
        Actor::Redactor,
        Actor::ToolProvider,
        Actor::Viewer,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::AgentLoop => "agentspec.session.AgentLoop",
            Actor::Delegate => "agentspec.session.Delegate",
            Actor::HarnessRuntime => "agentspec.session.HarnessRuntime",
            Actor::Human => "agentspec.session.Human",
            Actor::PeerSession => "agentspec.session.PeerSession",
            Actor::PermissionAuthority => "agentspec.session.PermissionAuthority",
            Actor::Redactor => "agentspec.session.Redactor",
            Actor::ToolProvider => "agentspec.session.ToolProvider",
            Actor::Viewer => "agentspec.view.Viewer",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::AgentLoop => &[
            "agentspec.session.EndTurn",
            "agentspec.session.RecordModelTurn",
            "agentspec.session.RequestToolCall",
            "agentspec.session.SubmitPrompt",
        ],
        Actor::Delegate => &[
            "agentspec.session.SubmitPrompt",
        ],
        Actor::HarnessRuntime => &[
            "agentspec.session.CompactContext",
            "agentspec.session.EndAgent",
            "agentspec.session.EndSession",
            "agentspec.session.OpenSession",
            "agentspec.session.StartAgent",
            "agentspec.session.SubmitPrompt",
        ],
        Actor::Human => &[
            "agentspec.session.DecideToolCall",
            "agentspec.session.SubmitPrompt",
        ],
        Actor::PeerSession => &[
            "agentspec.session.SubmitPrompt",
        ],
        Actor::PermissionAuthority => &[
            "agentspec.session.DecideToolCall",
        ],
        Actor::Redactor => &[
            "agentspec.session.RecordExposure",
        ],
        Actor::ToolProvider => &[
            "agentspec.session.CompleteToolCall",
        ],
        Actor::Viewer => &[],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}

/// The qualified names of the views `actor` may read, ordered by name: the views its grant
/// names. A view no actor's grant names is open to every caller and listed for none.
pub fn may_read(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::AgentLoop => &[],
        Actor::Delegate => &[],
        Actor::HarnessRuntime => &[],
        Actor::Human => &[],
        Actor::PeerSession => &[],
        Actor::PermissionAuthority => &[],
        Actor::Redactor => &[],
        Actor::ToolProvider => &[],
        Actor::Viewer => &[
            "agentspec.view.AgentTurns",
            "agentspec.view.OverviewFigures",
            "agentspec.view.SessionAgents",
            "agentspec.view.SessionById",
            "agentspec.view.SessionChecks",
            "agentspec.view.SessionEventCounts",
            "agentspec.view.SessionEvents",
            "agentspec.view.SessionList",
            "agentspec.view.SessionTimeline",
            "agentspec.view.SessionTurns",
        ],
    }
}

impl Caller {
    /// `true` when this caller may read `view`, a view some actor's grant names, named by
    /// its qualified name.
    pub fn may_read(&self, view: &str) -> bool {
        may_read(self.actor).contains(&view)
    }
}
