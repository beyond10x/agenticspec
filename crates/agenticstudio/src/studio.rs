//! The drafts of one studio, held in memory: each draft with the revisions of its architecture,
//! the patches applied to it and every application of a patch, applied or refused.

use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use agenticspec_patch::{Change, Patch, Refused};
use agenticspec_sim::check::Refusal;
use serde_json::{Number, Value};
use sha2::{Digest as _, Sha256};

use crate::types::architecture::{
    self as a, ArchitectureId, ArchitectureRevisionId, Digest, EssPresence as APresence,
    PatchApplicationId, PatchApplicationsRow, PatchId, PatchRule, PatchVerb, PatchesRow,
    RevisionsRow,
};
use crate::types::studio::{
    ArchitectureRef, DraftId, DraftsRow, EssPresence as SPresence, Name, RevisionRef, State,
};

/// The directory under the root that holds the architecture documents a draft opens.
pub const ARCHITECTURES: &str = "architectures";
/// The format an architecture document names.
pub const ARCHITECTURE_FORMAT: &str = "agenticspec-architecture/1";

/// Why the studio refused a request; the name is the error a client reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioError {
    /// `studio.draft.NoSuchDraft`.
    NoSuchDraft(String),
    /// `studio.draft.DraftNameTaken`.
    DraftNameTaken(String),
    /// The source is no architecture document under `architectures/`.
    NoSuchSource(String),
    /// The source is not an `agenticspec-architecture/1` document.
    NotAnArchitecture(String),
}

impl StudioError {
    pub fn name(&self) -> &'static str {
        match self {
            StudioError::NoSuchDraft(_) => "NoSuchDraft",
            StudioError::DraftNameTaken(_) => "DraftNameTaken",
            StudioError::NoSuchSource(_) => "NoSuchSource",
            StudioError::NotAnArchitecture(_) => "NotAnArchitecture",
        }
    }

    pub fn message(&self) -> String {
        match self {
            StudioError::NoSuchDraft(id) => format!("no draft has the id {id}"),
            StudioError::DraftNameTaken(name) => format!("an open draft is named {name}"),
            StudioError::NoSuchSource(s) => {
                format!("{s} is no architecture document under {ARCHITECTURES}/")
            }
            StudioError::NotAnArchitecture(m) => m.clone(),
        }
    }
}

/// One revision of a draft's architecture and the document it froze.
#[derive(Debug, Clone)]
pub struct Revision {
    pub row: RevisionsRow,
    pub document: Value,
}

#[derive(Debug, Clone)]
pub struct Draft {
    pub row: DraftsRow,
    /// In order; the last is the current one.
    pub revisions: Vec<Revision>,
    pub patches: Vec<PatchesRow>,
    pub applications: Vec<PatchApplicationsRow>,
}

impl Draft {
    pub fn current(&self) -> &Revision {
        // A draft is opened with its first revision and never loses one.
        &self.revisions[self.revisions.len() - 1]
    }
}

/// What applying a patch to a draft did.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// None when a rule's verb is no verb of `agenticspec-patch/1`: such a patch cannot be
    /// recorded, so neither can its application.
    pub patch: Option<PatchesRow>,
    pub application: Option<PatchApplicationsRow>,
    pub diff: Vec<Change>,
    /// The parts and links an applied patch changed; 0 for a refusal.
    pub changed_rows: usize,
    pub check: Vec<Refusal>,
    pub refusal: Option<Refused>,
    /// The revision an applied patch committed; none for a dry run or a refusal.
    pub revision: Option<RevisionsRow>,
}

pub struct Studio {
    root: PathBuf,
    drafts: Vec<Draft>,
    seed: u128,
    minted: u64,
}

fn number(n: usize) -> Number {
    Number::from(n as u64)
}

fn verb(name: &str) -> Option<PatchVerb> {
    Some(match name {
        "set" => PatchVerb::Set,
        "add" => PatchVerb::Add,
        "remove" => PatchVerb::Remove,
        "connect" => PatchVerb::Connect,
        "replicate" => PatchVerb::Replicate,
        "wrap" => PatchVerb::Wrap,
        "prepend" => PatchVerb::Prepend,
        "append" => PatchVerb::Append,
        "mark" => PatchVerb::Mark,
        "unmark" => PatchVerb::Unmark,
        _ => return None,
    })
}

impl Studio {
    /// A studio over the architecture documents under `root/architectures`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        Studio {
            root: root.into(),
            drafts: Vec::new(),
            seed,
            minted: 0,
        }
    }

    /// A new identifier in the form of a UUID, unique within this process.
    fn mint(&mut self, kind: &str) -> String {
        self.minted += 1;
        let hash = Sha256::digest(format!("{kind}:{}:{}", self.seed, self.minted).as_bytes());
        let mut b: [u8; 16] = [0; 16];
        b.copy_from_slice(&hash[..16]);
        b[6] = (b[6] & 0x0f) | 0x80;
        b[8] = (b[8] & 0x3f) | 0x80;
        let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
        format!(
            "{}-{}-{}-{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        )
    }

    /// The architecture documents under `architectures/`, by their path from the root, sorted.
    pub fn architectures(&self) -> Vec<String> {
        fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                let path = e.path();
                let rel = format!("{rel}/{name}");
                if path.is_dir() {
                    walk(&path, &rel, out);
                } else if name.ends_with(".yaml") {
                    out.push(rel);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.root.join(ARCHITECTURES), ARCHITECTURES, &mut out);
        out.sort();
        out
    }

    pub fn drafts(&self) -> &[Draft] {
        &self.drafts
    }

    pub fn draft(&self, id: &str) -> Result<&Draft, StudioError> {
        self.drafts
            .iter()
            .find(|d| d.row.draft_id.0 == id)
            .ok_or_else(|| StudioError::NoSuchDraft(id.to_string()))
    }

    fn name_taken(&self, name: &str) -> bool {
        self.drafts
            .iter()
            .any(|d| d.row.name.0 == name && d.row.state == State::Open)
    }

    /// Opens a draft of the architecture document `source` (a path under `architectures/`) at
    /// revision 1. Without a name the draft is named after its architecture, numbered when
    /// that name is taken.
    pub fn open(&mut self, source: &str, name: Option<&str>) -> Result<&Draft, StudioError> {
        let rel = Path::new(source);
        let inside = rel.components().all(|c| matches!(c, Component::Normal(_)))
            && rel.starts_with(ARCHITECTURES)
            && source.ends_with(".yaml");
        let path = self.root.join(rel);
        if !inside || !path.is_file() {
            return Err(StudioError::NoSuchSource(source.to_string()));
        }
        let document =
            agenticspec_sim::doc::load_yaml(&path).map_err(StudioError::NotAnArchitecture)?;
        if document["format"].as_str() != Some(ARCHITECTURE_FORMAT) {
            return Err(StudioError::NotAnArchitecture(format!(
                "{source} is not an {ARCHITECTURE_FORMAT} document"
            )));
        }
        let name = match name {
            Some(n) if self.name_taken(n) => return Err(StudioError::DraftNameTaken(n.into())),
            Some(n) => n.to_string(),
            None => {
                let base = document["architecture"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| rel.file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "draft".into());
                let mut n = base.clone();
                let mut i = 1;
                while self.name_taken(&n) {
                    i += 1;
                    n = format!("{base} {i}");
                }
                n
            }
        };
        let architecture = self.mint("architecture");
        let revision = self.mint("revision");
        let draft_id = self.mint("draft");
        let first = Revision {
            row: RevisionsRow {
                revision_id: ArchitectureRevisionId(revision.clone()),
                architecture_id: ArchitectureId(architecture.clone()),
                number: number(1),
                digest: Digest(agenticspec_patch::digest(&document)),
            },
            document,
        };
        self.drafts.push(Draft {
            row: DraftsRow {
                draft_id: DraftId(draft_id),
                name: Name(name),
                architecture: ArchitectureRef(architecture),
                source: SPresence::Present(source.to_string()),
                opened_at_revision: SPresence::Present(RevisionRef(revision)),
                state: State::Open,
            },
            revisions: vec![first],
            patches: Vec::new(),
            applications: Vec::new(),
        });
        let last = self.drafts.len() - 1;
        Ok(&self.drafts[last])
    }

    /// Applies `patch` to the draft's current revision. Applied and not a dry run: a new
    /// revision. Otherwise the draft keeps its revision; the application is recorded either way
    /// when the patch can be.
    pub fn apply(
        &mut self,
        id: &str,
        patch: &Patch,
        dry_run: bool,
    ) -> Result<Outcome, StudioError> {
        let index = self
            .drafts
            .iter()
            .position(|d| d.row.draft_id.0 == id)
            .ok_or_else(|| StudioError::NoSuchDraft(id.to_string()))?;
        let current = self.drafts[index].current().clone();
        let result = agenticspec_patch::apply(&current.document, patch);

        let rules: Option<Vec<PatchRule>> = patch
            .rules
            .iter()
            .map(|r| {
                let v = verb(agenticspec_patch::verb_of(r)?)?;
                Some(PatchRule {
                    verb: v,
                    text: r.clone(),
                })
            })
            .collect();
        let patch_row = match rules {
            Some(rules) => {
                let row = PatchesRow {
                    patch_id: PatchId(self.mint("patch")),
                    name: patch.name.as_ref().map_or(APresence::Absent, |n| {
                        APresence::Present(a::Name(n.clone()))
                    }),
                    rules,
                    digest: Digest(patch.digest()),
                };
                Some(row)
            }
            None => None,
        };
        let architecture_id = ArchitectureId(self.drafts[index].row.architecture.0.clone());
        let mut outcome = Outcome {
            patch: patch_row.clone(),
            application: None,
            diff: Vec::new(),
            changed_rows: 0,
            check: Vec::new(),
            refusal: None,
            revision: None,
        };
        let mut application = patch_row.as_ref().map(|p| PatchApplicationsRow {
            application_id: PatchApplicationId(String::new()),
            architecture_id: architecture_id.clone(),
            patch_id: p.patch_id.clone(),
            dry_run,
            revision_id: APresence::Absent,
            changed_rows: APresence::Absent,
            rule_index: APresence::Absent,
            error: APresence::Absent,
            state: a::State::Requested,
        });
        match result {
            Ok(applied) => {
                outcome.diff = applied.diff;
                outcome.changed_rows = applied.changed_rows;
                if let Some(app) = application.as_mut() {
                    app.changed_rows = APresence::Present(number(applied.changed_rows));
                    app.state = a::State::Applied;
                }
                if !dry_run {
                    let number_next = self.drafts[index].revisions.len() + 1;
                    let row = RevisionsRow {
                        revision_id: ArchitectureRevisionId(self.mint("revision")),
                        architecture_id,
                        number: number(number_next),
                        digest: Digest(agenticspec_patch::digest(&applied.document)),
                    };
                    if let Some(app) = application.as_mut() {
                        app.revision_id = APresence::Present(row.revision_id.clone());
                    }
                    self.drafts[index].revisions.push(Revision {
                        row: row.clone(),
                        document: applied.document,
                    });
                    outcome.revision = Some(row);
                }
            }
            Err(refused) => {
                outcome.diff = refused.diff.clone();
                outcome.check = refused.check.clone();
                if let Some(app) = application.as_mut() {
                    app.rule_index = refused
                        .rule
                        .map_or(APresence::Absent, |n| APresence::Present(number(n)));
                    app.error = APresence::Present(refused.error.name().to_string());
                    app.state = a::State::Refused;
                }
                outcome.refusal = Some(refused);
            }
        }
        if let (Some(mut app), Some(p)) = (application, patch_row) {
            app.application_id = PatchApplicationId(self.mint("application"));
            let draft = &mut self.drafts[index];
            draft.patches.push(p);
            draft.applications.push(app.clone());
            outcome.application = Some(app);
        }
        Ok(outcome)
    }
}
