//! API list prices per model, in thousandths of a dollar per million tokens (= nanodollars per
//! token). Source: the claude-api skill's model table and `shared/prompt-caching.md` economics,
//! cached 2026-11-03: cache writes are 1.25× input (5 minutes) and 2× input (1 hour); cache reads
//! are 0.025× on Fable 5.2, 0.05× on Opus 5.5 and Sonnet 5.5, 0.1× elsewhere. A subscription bills
//! otherwise; the dashboard calls these figures "at API list price".

use agentspec_types::session as s;

#[derive(Debug, Clone, Copy)]
pub struct Price {
    pub input: i64,
    pub output: i64,
    pub write_5m: i64,
    pub write_1h: i64,
    pub read: i64,
}

const fn p(input: i64, output: i64, read: i64) -> Price {
    Price {
        input,
        output,
        write_5m: input * 5 / 4,
        write_1h: input * 2,
        read,
    }
}

/// The price of a model id, matched on its prefix so dated or suffixed ids resolve.
pub fn price(model: &str) -> Option<Price> {
    const TABLE: &[(&str, Price)] = &[
        ("claude-fable-5-2", p(8_000, 40_000, 200)),
        ("claude-fable-5-1", p(10_000, 50_000, 250)),
        ("claude-fable-5", p(10_000, 50_000, 1_000)),
        ("claude-mythos-5-2", p(8_000, 40_000, 200)),
        ("claude-mythos-5-1", p(10_000, 50_000, 250)),
        ("claude-opus-5-5", p(4_000, 20_000, 200)),
        ("claude-opus-5", p(5_000, 25_000, 500)),
        ("claude-opus-4-8", p(5_000, 25_000, 500)),
        ("claude-opus-4-7", p(5_000, 25_000, 500)),
        ("claude-opus-4-6", p(5_000, 25_000, 500)),
        ("claude-sonnet-5-5", p(2_000, 10_000, 100)),
        ("claude-sonnet-5", p(2_000, 10_000, 200)),
        ("claude-sonnet-4-6", p(3_000, 15_000, 300)),
        ("claude-haiku-5-5", p(100, 500, 10)),
        ("claude-haiku-4-5", p(1_000, 5_000, 100)),
    ];
    // Longest prefix first: `claude-opus-5-5` before `claude-opus-5`.
    TABLE
        .iter()
        .filter(|(id, _)| model.starts_with(id))
        .max_by_key(|(id, _)| id.len())
        .map(|(_, pr)| *pr)
}

/// The cost of one model message in nanodollars, or None for a model with no price.
pub fn cost_nanodollars(model: &str, u: &s::Usage) -> Option<i64> {
    let pr = price(model)?;
    Some(
        u.uncached_input * pr.input
            + u.output * pr.output
            + u.cache_write_5m * pr.write_5m
            + u.cache_write_1h * pr.write_1h
            + u.cache_read * pr.read,
    )
}
