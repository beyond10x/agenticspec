//! The layer map: blocks as cards in rows by layer (the highest layer on top, the foundation at
//! the bottom), placed on a column grid so that dependencies run as straight as they can, and the
//! dependency arrows routed between the cards.

use crate::model::Block;
use std::collections::BTreeMap;

pub const WIDTH: f64 = 980.0;
const LABEL: f64 = 104.0;
const GUTTER: f64 = 30.0;
const CARD_H: f64 = 92.0;
const BAND_H: f64 = 64.0;
const ROW_GAP: f64 = 50.0;
const TOP: f64 = 6.0;
const BOTTOM: f64 = 6.0;

#[derive(Debug, Clone)]
pub struct Card {
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub count: usize,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub layer: u32,
    pub name: String,
    pub y: f64,
    pub h: f64,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub width: f64,
    pub height: f64,
    pub rows: Vec<Row>,
    pub cards: Vec<Card>,
    pub edges: Vec<Edge>,
}

fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Every injective placement of `k` items on `n` slots, in lexicographic order.
fn placements(k: usize, n: usize) -> Vec<Vec<usize>> {
    fn go(k: usize, n: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for s in 0..n {
            if !cur.contains(&s) {
                cur.push(s);
                go(k, n, cur, out);
                cur.pop();
            }
        }
    }
    let mut out = Vec::new();
    go(k, n, &mut Vec::new(), &mut out);
    out
}

/// `edges` are (from, to, count) between block indexes; the foundation layer (0) is drawn as one
/// band and gets no arrows, since every block uses it.
pub fn layout(blocks: &[Block], edges: &[(usize, usize, usize)]) -> Map {
    let top = blocks.iter().map(|b| b.layer).max().unwrap_or(0);
    let mut by_layer: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, b) in blocks.iter().enumerate() {
        by_layer.entry(b.layer).or_default().push(i);
    }
    let columns = by_layer
        .iter()
        .filter(|(l, _)| **l > 0)
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let card_w = (WIDTH - LABEL - GUTTER * (columns as f64 + 1.0)) / columns as f64;
    let drawn: Vec<(usize, usize, usize)> = edges
        .iter()
        .copied()
        .filter(|(f, t, _)| blocks[*f].layer > 0 && blocks[*t].layer > 0)
        .collect();

    // Columns: start centred, then sweep the rows, each taking the placement that keeps its
    // dependencies straightest against every other row.
    let mut slot = vec![0usize; blocks.len()];
    for (layer, members) in &by_layer {
        if *layer == 0 {
            continue;
        }
        let start = (columns - members.len()) / 2;
        for (k, i) in members.iter().enumerate() {
            slot[*i] = start + k;
        }
    }
    let layers: Vec<u32> = by_layer.keys().copied().filter(|l| *l > 0).collect();
    for sweep in 0..8 {
        let order: Vec<u32> = if sweep % 2 == 0 {
            layers.iter().rev().copied().collect()
        } else {
            layers.clone()
        };
        for layer in order {
            let members = &by_layer[&layer];
            let mut best: Option<(i64, Vec<usize>)> = None;
            for p in placements(members.len(), columns) {
                let at = |i: usize| {
                    members
                        .iter()
                        .position(|m| *m == i)
                        .map_or(slot[i], |k| p[k])
                };
                // A small pull to the middle keeps a short row from hugging one side.
                let mut cost: i64 = p
                    .iter()
                    .map(|s| (2 * *s as i64 - (columns as i64 - 1)).abs())
                    .sum();
                for (f, t, _) in &drawn {
                    let (f, t) = (*f, *t);
                    if !members.contains(&f) && !members.contains(&t) {
                        continue;
                    }
                    let d = (at(f) as i64 - at(t) as i64).abs();
                    if blocks[f].layer == blocks[t].layer {
                        cost += 16 * (d - 1).max(0);
                    } else {
                        let span = blocks[f].layer.abs_diff(blocks[t].layer) as i64;
                        cost += 4 * d * if span == 1 { 3 } else { 1 };
                    }
                }
                if best.as_ref().is_none_or(|(c, _)| cost < *c) {
                    best = Some((cost, p));
                }
            }
            if let Some((_, p)) = best {
                for (k, i) in members.iter().enumerate() {
                    slot[*i] = p[k];
                }
            }
        }
    }

    let row_y = |layer: u32| TOP + (top - layer) as f64 * (CARD_H + ROW_GAP);
    let col_x = |c: usize| LABEL + GUTTER + c as f64 * (card_w + GUTTER);
    let gutter_x = |g: usize| LABEL + GUTTER / 2.0 + g as f64 * (card_w + GUTTER);
    let mut cards = Vec::new();
    for (i, b) in blocks.iter().enumerate() {
        if b.layer == 0 {
            cards.push(Card {
                index: i,
                x: r1(col_x(0)),
                y: r1(row_y(0)),
                w: r1(columns as f64 * card_w + (columns as f64 - 1.0) * GUTTER),
                h: BAND_H,
            });
        } else {
            cards.push(Card {
                index: i,
                x: r1(col_x(slot[i])),
                y: r1(row_y(b.layer)),
                w: r1(card_w),
                h: CARD_H,
            });
        }
    }
    let card = |i: usize| cards.iter().find(|c| c.index == i).expect("card");
    let mut rows: Vec<Row> = by_layer
        .iter()
        .map(|(layer, members)| Row {
            layer: *layer,
            name: blocks[members[0]].layer_name().to_string(),
            y: r1(row_y(*layer)),
            h: if *layer == 0 { BAND_H } else { CARD_H },
        })
        .collect();
    rows.reverse();

    // Arrows in a fixed order: from the top row down, left to right.
    let mut order: Vec<(usize, usize, usize)> = drawn.clone();
    order.sort_by_key(|(f, t, _)| {
        (
            std::cmp::Reverse(blocks[*f].layer),
            slot[*f],
            std::cmp::Reverse(blocks[*t].layer),
            slot[*t],
        )
    });
    // A long arrow runs down a gutter between two columns; pick the least used one between them.
    let mut gutter_of: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut load = vec![0usize; columns + 1];
    for (f, t, _) in &order {
        let (bf, bt) = (&blocks[*f], &blocks[*t]);
        if bf.layer == bt.layer || bf.layer - bt.layer == 1 {
            continue;
        }
        let (a, b) = (slot[*f].min(slot[*t]), slot[*f].max(slot[*t]));
        let candidates: Vec<usize> = if a == b {
            vec![a, a + 1]
        } else {
            (a + 1..=b).collect()
        };
        let g = *candidates
            .iter()
            .min_by_key(|g| (load[**g], g.abs_diff(slot[*t])))
            .expect("a gutter");
        load[g] += 1;
        gutter_of.insert((*f, *t), g);
    }
    // Lanes inside each gutter.
    let mut lane: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for g in 0..=columns {
        let mut users: Vec<(usize, usize)> = gutter_of
            .iter()
            .filter(|(_, x)| **x == g)
            .map(|(k, _)| *k)
            .collect();
        users.sort_by_key(|(f, t)| (std::cmp::Reverse(blocks[*f].layer), blocks[*t].layer));
        let n = users.len();
        let step = if n > 1 {
            ((GUTTER - 10.0) / (n as f64 - 1.0)).min(6.0)
        } else {
            0.0
        };
        for (k, e) in users.into_iter().enumerate() {
            lane.insert(e, (k as f64 - (n as f64 - 1.0) / 2.0) * step);
        }
    }
    // The x a card's arrow leaves from or arrives at, spread over its edge by the other end's x.
    let toward = |f: usize, t: usize, at_source: bool| -> f64 {
        match gutter_of.get(&(f, t)) {
            Some(g) => gutter_x(*g) + lane[&(f, t)],
            None => {
                let other = if at_source { card(t) } else { card(f) };
                other.x + other.w / 2.0
            }
        }
    };
    let mut ports: BTreeMap<(usize, usize, bool), f64> = BTreeMap::new();
    for i in 0..blocks.len() {
        for at_source in [true, false] {
            let mut mine: Vec<(f64, usize, usize)> = order
                .iter()
                .filter(|(f, t, _)| blocks[*f].layer != blocks[*t].layer)
                .filter(|(f, t, _)| if at_source { *f == i } else { *t == i })
                .map(|(f, t, _)| (toward(*f, *t, at_source), *f, *t))
                .collect();
            mine.sort_by(|a, b| a.0.total_cmp(&b.0).then((a.1, a.2).cmp(&(b.1, b.2))));
            let c = card(i);
            let n = mine.len() as f64;
            let span = (c.w - 48.0).min(14.0 * n);
            for (k, (_, f, t)) in mine.into_iter().enumerate() {
                let x = c.x + c.w / 2.0 - span / 2.0 + span * (k as f64 + 0.5) / n;
                ports.insert((f, t, at_source), x);
            }
        }
    }
    let mut out = Vec::new();
    for (f, t, count) in order {
        let (cf, ct) = (card(f), card(t));
        let path = if blocks[f].layer == blocks[t].layer {
            let y = cf.y + cf.h / 2.0;
            if cf.x < ct.x {
                format!("M{} {} L{} {}", r1(cf.x + cf.w), r1(y), r1(ct.x), r1(y))
            } else {
                format!("M{} {} L{} {}", r1(cf.x), r1(y), r1(ct.x + ct.w), r1(y))
            }
        } else {
            let sx = ports[&(f, t, true)];
            let tx = ports[&(f, t, false)];
            let sy = cf.y + cf.h;
            let ty = ct.y;
            match gutter_of.get(&(f, t)) {
                None => {
                    let m = (ty - sy) / 2.0;
                    format!(
                        "M{} {} C{} {} {} {} {} {}",
                        r1(sx),
                        r1(sy),
                        r1(sx),
                        r1(sy + m),
                        r1(tx),
                        r1(ty - m),
                        r1(tx),
                        r1(ty)
                    )
                }
                Some(g) => {
                    let gx = gutter_x(*g) + lane[&(f, t)];
                    let h = ROW_GAP / 2.0;
                    format!(
                        "M{} {} C{} {} {} {} {} {} L{} {} C{} {} {} {} {} {}",
                        r1(sx),
                        r1(sy),
                        r1(sx),
                        r1(sy + h * 0.7),
                        r1(gx),
                        r1(sy + h * 0.3),
                        r1(gx),
                        r1(sy + h),
                        r1(gx),
                        r1(ty - h),
                        r1(gx),
                        r1(ty - h * 0.3),
                        r1(tx),
                        r1(ty - h * 0.7),
                        r1(tx),
                        r1(ty)
                    )
                }
            }
        };
        out.push(Edge {
            from: f,
            to: t,
            count,
            path,
        });
    }
    let height = TOP + top as f64 * (CARD_H + ROW_GAP) + BAND_H + BOTTOM;
    Map {
        width: WIDTH,
        height: r1(height),
        rows,
        cards,
        edges: out,
    }
}
