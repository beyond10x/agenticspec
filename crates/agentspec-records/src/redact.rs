//! The redaction pass: finds secrets and personal data in content, never keeps the value, and
//! names each distinct value by a per-session alias (`gitlab_token#1`).

use std::collections::HashMap;
use std::sync::OnceLock;

use agentspec_types::session::{ExposureChannel, ExposureClass, Verdict};
use regex::Regex;
use sha2::{Digest, Sha256};

/// One match, without its value.
pub struct Finding {
    pub kind: &'static str,
    pub class: ExposureClass,
    pub verdict: Verdict,
    pub alias: String,
    pub channel: ExposureChannel,
}

struct Detector {
    kind: &'static str,
    class: ExposureClass,
    pattern: Regex,
}

fn detectors() -> &'static [Detector] {
    static D: OnceLock<Vec<Detector>> = OnceLock::new();
    D.get_or_init(|| {
        let secret = |kind, p: &str| Detector {
            kind,
            class: ExposureClass::Secret,
            pattern: Regex::new(p).unwrap(),
        };
        vec![
            secret("aws_access_key", r"AKIA[0-9A-Z]{16}"),
            secret("private_key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
            secret("slack_token", r"xox[baprs]-[0-9A-Za-z-]{10,}"),
            secret("github_token", r"gh[pousr]_[0-9A-Za-z]{30,}"),
            secret("gitlab_token", r"glpat-[0-9A-Za-z_-]{16,}"),
            secret("anthropic_key", r"sk-ant-[0-9A-Za-z_-]{20,}"),
            secret(
                "jwt",
                r"eyJ[0-9A-Za-z_-]{10,}\.eyJ[0-9A-Za-z_-]{10,}\.[0-9A-Za-z_-]{10,}",
            ),
            secret("bearer", r"[Bb]earer [0-9A-Za-z._~+/-]{24,}"),
            Detector {
                kind: "email",
                class: ExposureClass::PersonalData,
                pattern: Regex::new(
                    r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}",
                )
                .unwrap(),
            },
        ]
    })
}

/// A value that is a known fixture or an address nobody owns is `Ok`; every other match is `Unk`.
/// `Gap` is only ever set by a confirmation, which this pass does not make.
fn verdict(kind: &str, value: &str) -> Verdict {
    let lower = value.to_ascii_lowercase();
    let fixture = [
        "example",
        "xxxx",
        "dummy",
        "fake",
        "placeholder",
        "redacted",
        "abcdef",
        "1234567",
    ];
    if fixture.iter().any(|f| lower.contains(f)) {
        return Verdict::Ok;
    }
    if kind == "email" {
        let domain = lower.rsplit('@').next().unwrap_or("");
        let nobody = [
            "example.com",
            "example.org",
            "example.net",
            "test",
            "localhost",
            "invalid",
        ];
        if nobody
            .iter()
            .any(|n| domain == *n || domain.ends_with(&format!(".{n}")))
            || domain.ends_with("noreply.github.com")
            || lower.starts_with("noreply@")
            || lower.starts_with("no-reply@")
        {
            return Verdict::Ok;
        }
    }
    Verdict::Unk
}

#[derive(Default)]
pub struct Redactor {
    aliases: HashMap<[u8; 32], String>,
    counters: HashMap<&'static str, usize>,
}

impl Redactor {
    pub fn scan(&mut self, text: &str, channel: ExposureChannel) -> Vec<Finding> {
        let mut out = Vec::new();
        if text.len() < 8 {
            return out;
        }
        for d in detectors() {
            for m in d.pattern.find_iter(text) {
                let value = m.as_str();
                let key: [u8; 32] = Sha256::digest(value.as_bytes()).into();
                let alias = match self.aliases.get(&key) {
                    Some(a) => a.clone(),
                    None => {
                        let n = self.counters.entry(d.kind).or_insert(0);
                        *n += 1;
                        let a = format!("{}#{}", d.kind, n);
                        self.aliases.insert(key, a.clone());
                        a
                    }
                };
                out.push(Finding {
                    kind: d.kind,
                    class: d.class,
                    verdict: verdict(d.kind, value),
                    alias,
                    channel,
                });
            }
        }
        out
    }
}

/// sha256 of the bytes, lower-case hex.
pub fn digest_hex(bytes: &[u8]) -> String {
    let d = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
