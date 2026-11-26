//! The dashboard file: one self-contained HTML page with the `sessionspec.cli.Dashboard` document
//! embedded as JSON. It loads nothing from the network.

use sessionspec_types::cli::Dashboard;

const TEMPLATE: &str = include_str!("../assets/dashboard.html");
const SLOT: &str = "__SESSIONSPEC_DASHBOARD__";

/// The page for this document. The JSON is written by the generated wire encoder; every `<` is
/// written `\u003c`, so no value can close or comment out the script element it sits in.
pub fn render(dashboard: &Dashboard) -> String {
    let mut json = String::new();
    sessionspec_server::wire::encode_sessionspec_cli_dashboard(dashboard, &mut json);
    TEMPLATE.replacen(SLOT, &json.replace('<', "\\u003c"), 1)
}
