//! One self-contained HTML page for a graph, in one visual language.
//!
//! - [`page`]: the render model `agenticspec-render-model/1` ([`page::Page`]: nodes, ports, edges,
//!   regions, marks and panels in neutral terms) and its entry point [`page::render`], which
//!   returns one complete HTML page. Another repository draws its own graphs through it.
//! - [`architecture`]: an agenticspec architecture (`agenticspec-architecture/1`) and, when
//!   given, the result of its optimization (`agenticspec-sim-result/1`), mapped onto a page and
//!   drawn through the same renderer.
//!
//! A page holds its CSS, its script and its SVG inline, embeds its input as JSON, and loads
//! nothing from the network. The output is a function of the input only.

pub mod architecture;
mod fmt;
mod graph;
pub mod page;
