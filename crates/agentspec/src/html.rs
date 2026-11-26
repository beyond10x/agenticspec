//! The dashboard file: one self-contained HTML page with the `agentspec.cli.Dashboard` document
//! embedded as JSON. It loads nothing from the network.

use agentspec_types::cli::Dashboard;

const TEMPLATE: &str = include_str!("../assets/dashboard.html");
const SLOT: &str = "__AGENTSPEC_DASHBOARD__";

/// The page for this document. The JSON is written by the generated wire encoder; every `<` is
/// written `\u003c`, so no value can close or comment out the script element it sits in.
pub fn render(dashboard: &Dashboard) -> String {
    let mut json = String::new();
    agentspec_server::wire::encode_agentspec_cli_dashboard(dashboard, &mut json);
    TEMPLATE.replacen(SLOT, &json.replace('<', "\\u003c"), 1)
}
