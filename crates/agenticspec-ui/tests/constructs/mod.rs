//! Constructs of released `ess-ui/1` the generator draws beyond its first set (R108): a read's row
//! predicate, a collection's sort and groups, cards, a poll interval, a page's actor, the header's
//! help and metrics, an action shown as an icon or behind a confirm, and a form's choice. Each is
//! held by what the generated page carries; what is still not drawn is refused by its node path.

use super::*;

/// A document with one page `p` whose state is `owner`, `search` and `every`, whose shell has the
/// state `mine`, and whose sections are `sections` (YAML, indented by six). `extra` goes into the
/// page (indented by four).
fn doc(extra: &str, sections: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: t\nplacement_profile: fat\nshells:\n  app:\n    regions: {{main: {{kind: page_outlet}}}}\n    state: {{mine: {{type: boolean, class: preference}}}}\nnavigation: {{home: p, sections: [{{name: s, pages: [p]}}]}}\npage_kinds: {{plain: {{purpose: plain}}}}\npages:\n  p:\n    kind: plain\n    state:\n      owner: {{type: string, class: page_state}}\n      search: {{type: string, class: page_state}}\n      every: {{type: string, class: page_state}}\n{extra}    sections:\n{sections}"
    )
}

fn page(text: &str) -> String {
    match generate(text) {
        Ok(files) => files["pages/PPage.vue"].clone(),
        Err(r) => panic!("refused: {r:?}\n{text}"),
    }
}

/// A collection over `t.V` named `list` with a filter.
fn filtered(filter: &str) -> String {
    doc(
        "",
        &format!(
            "      - name: list\n        component: collection\n        reads: {{view: t.V, filter: \"{filter}\"}}\n        columns: [title, state]\n"
        ),
    )
}

mod filter {
    use super::*;

    #[test]
    fn a_comparison_joined_by_and_and_or_compiles_to_the_row_predicate() {
        let p = page(&filtered(
            "row.state == open and (row.owner == state.owner or state.owner == null)",
        ));
        assert!(
            p.contains("r_list.rows.filter((row) => (same(row['state'], 'open') && (same(row['owner'], page.state['owner']) || same(page.state['owner'], null))))"),
            "{p}"
        );
        assert!(p.contains("same, ") || p.contains("same }"), "{p}");
    }

    #[test]
    fn inequality_membership_negation_and_text_search_compile() {
        let p = page(&filtered(
            "row.state != open and not (row.priority in [low, 'normal', 3]) and row.title contains state.search",
        ));
        assert!(
            p.contains("((!same(row['state'], 'open') && !member(row['priority'], ['low', 'normal', 3])) && contains(row['title'], page.state['search']))"),
            "{p}"
        );
    }

    #[test]
    fn a_nested_path_shell_state_and_a_bare_negated_path_compile() {
        let p = page(&filtered("row.owner.name == shell.mine or not row.done"));
        assert!(
            p.contains(
                "(same(at(row, ['owner', 'name']), shell.state['mine']) || !truthy(row['done']))"
            ),
            "{p}"
        );
    }

    #[test]
    fn two_panels_of_one_view_with_different_filters_stay_two_sections() {
        let text = doc(
            "",
            "      - {name: open, title: Open, component: collection, reads: {view: t.V, filter: row.state == open}, columns: [title]}\n      - {name: done, title: Done, component: collection, reads: {view: t.V, filter: row.state == done}, columns: [title]}\n",
        );
        let p = page(&text);
        assert!(p.contains("<KitPanel name=\"open\""), "{p}");
        assert!(p.contains("<KitPanel name=\"done\""), "{p}");
        assert!(
            p.contains("r_open.rows.filter((row) => same(row['state'], 'open'))"),
            "{p}"
        );
        assert!(
            p.contains("r_done.rows.filter((row) => same(row['state'], 'done'))"),
            "{p}"
        );
    }

    #[test]
    fn a_text_search_draws_the_search_box_of_its_state() {
        let p = page(&filtered("row.title contains state.search"));
        assert!(p.contains("<KitSearch label=\"Find in list\""), "{p}");
    }

    /// The grammar is the one `ess ui check` 0.99.2 holds a filter to: what it accepts generates,
    /// what it refuses is refused (each case was run through `ess ui check`).
    #[test]
    fn the_grammar_accepts_what_ess_ui_check_accepts_and_no_more() {
        for ok in [
            "row.state == 'open now'",
            "not row.done",
            "row.state in [open, done]",
            "row.owner.name == x",
            "row.n == 3.5",
            "row.n != -1",
            "(row.state == open)",
            "row.done and row.state == open",
            "row.state == open or not (row.a == b and row.c != d)",
            "row.tags contains open",
        ] {
            let text = filtered(ok);
            assert!(generate(&text).is_ok(), "{ok}: {:?}", generate(&text).err());
        }
        for bad in [
            "row.n >= 3",
            "row.state = open",
            "contains(row.a, x)",
            "row.n == -1.5e3",
            "row.state ==",
            "(row.state == open",
        ] {
            assert_refused_at(
                &filtered(bad),
                "pages/p/sections/list/reads/filter",
                "comparison, membership",
            );
        }
    }

    #[test]
    fn and_binds_tighter_than_or() {
        let p = page(&filtered("row.a == 1 or row.b == 2 and row.c == 3"));
        assert!(
            p.contains("(same(row['a'], 1) || (same(row['b'], 2) && same(row['c'], 3)))"),
            "{p}"
        );
    }

    #[test]
    fn a_filter_ess_ui_refuses_is_refused_by_its_path() {
        let at = "pages/p/sections/list/reads/filter";
        assert_refused_at(&filtered("row.n > 3"), at, "comparison, membership");
        assert_refused_at(&filtered("row.state"), at, "boolean");
        assert_refused_at(
            &filtered("row.owner == actor.user_id"),
            at,
            "row, params, state, shell",
        );
        assert_refused_at(&filtered("row.state == state.nowhere"), at, "state.nowhere");
        assert_refused_at(
            &filtered("row.state in state.owner"),
            at,
            "comparison, membership",
        );
    }
}

mod sort_and_groups {
    use super::*;

    fn collection(extra: &str) -> String {
        doc(
            "",
            &format!(
                "      - name: list\n        component: collection\n        reads: {{view: t.V, filter: row.state != done}}\n        columns: [title, {{field: n, as: number}}, priority]\n{extra}"
            ),
        )
    }

    #[test]
    fn a_sort_orders_the_rows_after_the_filter() {
        let p = page(&collection("        sort: {by: opened_at, dir: desc}\n"));
        assert!(
            p.contains("const rows_list = computed(() => sortRows(r_list.rows.filter((row) => !same(row['state'], 'done')), 'opened_at', 'desc', false));"),
            "{p}"
        );
        assert!(p.contains(":rows=\"rows_list\""), "{p}");
    }

    #[test]
    fn a_sort_by_a_number_column_sorts_numbers_and_ascends_by_default() {
        let p = page(&collection("        sort: {by: n}\n"));
        assert!(p.contains("'n', 'asc', true)"), "{p}");
    }

    #[test]
    fn a_sort_the_generator_does_not_draw_is_refused_by_its_path() {
        assert_refused_at(
            &collection("        sort: {by: n, mode: server}\n"),
            "pages/p/sections/list/sort/mode",
            "server",
        );
        assert_refused_at(
            &collection("        sort: {by: n, allowed: [n, title]}\n"),
            "pages/p/sections/list/sort/allowed",
            "unknown property",
        );
    }

    #[test]
    fn groups_split_the_rows_under_their_value_in_the_order_given() {
        let p = page(&collection(
            "        sort: {by: n}\n        group_by: priority\n        group_order: [high, normal, low]\n",
        ));
        assert!(
            p.contains("const groups_list = computed(() => groupRows(rows_list.value, 'priority', ['high', 'normal', 'low'], false));"),
            "{p}"
        );
        assert!(
            p.contains("<KitPanel v-for=\"g in groups_list\" :key=\"g.name\" :name=\"g.name\" :title=\"g.name\" bare>"),
            "{p}"
        );
        assert!(p.contains(":rows=\"g.rows\""), "{p}");
    }

    #[test]
    fn empty_groups_are_shown_when_asked() {
        let p = page(&collection(
            "        group_by: priority\n        group_order: [high]\n        show_empty_groups: true\n",
        ));
        assert!(p.contains("'priority', ['high'], true)"), "{p}");
    }

    #[test]
    fn a_group_order_without_a_group_is_refused() {
        assert_refused_at(
            &collection("        group_order: [high]\n"),
            "pages/p/sections/list/group_order",
            "group_by",
        );
    }
}

mod more {
    use super::*;

    #[test]
    fn cards_are_drawn_as_the_kits_list() {
        let p = page(&doc(
            "",
            "      - {name: list, component: collection, style: cards, reads: t.V, columns: [title]}\n",
        ));
        assert!(p.contains("<KitList label=\"list\""), "{p}");
    }

    #[test]
    fn a_poll_interval_reads_again_every_so_often() {
        let p = page(&doc(
            "",
            "      - {name: list, component: collection, reads: {view: t.V, refresh: 30s}, columns: [title]}\n",
        ));
        assert!(p.contains("usePoll(r_list, 30000);"), "{p}");
        assert_refused_at(
            &doc(
                "",
                "      - {name: list, component: collection, reads: {view: t.V, refresh: state.every}, columns: [title]}\n",
            ),
            "pages/p/sections/list/reads/refresh",
            "duration",
        );
    }

    #[test]
    fn a_page_actor_is_accepted_and_changes_nothing_drawn() {
        let sections =
            "      - {name: list, component: collection, reads: t.V, columns: [title]}\n";
        assert_eq!(
            page(&doc("    actor: t.Agent\n", sections)),
            page(&doc("", sections))
        );
    }

    #[test]
    fn the_header_help_is_drawn_in_the_bar() {
        let p = page(&doc(
            "    header:\n      help: {text: What the list holds, link: \"https://example.org/help\"}\n",
            "      - {name: list, component: collection, reads: t.V, columns: [title]}\n",
        ));
        assert!(p.contains("<KitToolbar label=\"Help\">"), "{p}");
        assert!(p.contains("What the list holds"), "{p}");
        assert!(
            p.contains("const help_link = 'https://example.org/help';"),
            "{p}"
        );
        assert!(
            p.contains("<KitLink :href=\"help_link\">Help</KitLink>"),
            "{p}"
        );
    }

    #[test]
    fn the_header_metrics_are_drawn_from_their_reads() {
        let p = page(&doc(
            "    header:\n      metrics:\n        - {name: open, component: metric, reads: t.V, aggregate: count, label: Open}\n        - {name: load, component: metric, reads: t.W, aggregate: sum, field: n, label: Load}\n",
            "      - {name: list, component: collection, reads: t.V, columns: [title]}\n",
        ));
        assert!(p.contains("<KitToolbar label=\"Metrics\">"), "{p}");
        assert!(
            p.contains(
                "<KitMetric label=\"Open\" :value=\"metricValue(r_p_open, 'count', null, null)\" />"
            ),
            "{p}"
        );
        assert!(
            p.contains(
                "<KitMetric label=\"Load\" :value=\"metricValue(r_p_load, 'sum', 'n', null)\" />"
            ),
            "{p}"
        );
        assert_refused_at(
            &doc(
                "    header:\n      metrics:\n        - {name: open, component: metric, from: channel.c.n, label: Open}\n",
                "      - {name: list, component: collection, reads: t.V, columns: [title]}\n",
            ),
            "pages/p/header/metrics/open/from",
            "unknown property",
        );
    }

    #[test]
    fn an_assistant_region_is_refused_by_its_path() {
        let text = doc(
            "",
            "      - {name: list, component: collection, reads: t.V, columns: [title]}\n",
        )
        .replace(
            "regions: {main: {kind: page_outlet}}",
            "regions: {main: {kind: page_outlet}, helper: {kind: assistant}}",
        );
        assert_refused_at(&text, "shells/app/regions/helper/kind", "assistant");
    }

    #[test]
    fn a_row_action_as_an_icon_is_a_compact_button() {
        let p = page(&doc(
            "",
            "      - name: list\n        component: collection\n        reads: t.V\n        columns: [title]\n        row_actions: [{name: close, does: t.Close, bind: {id: row.id}, as: icon, label: Close}]\n",
        ));
        assert!(
            p.contains("<KitButton v-if=\"may.allows('t.Close')\" tone=\"ghost\" size=\"sm\""),
            "{p}"
        );
    }

    #[test]
    fn a_confirm_asks_first_and_its_show_condition_decides_when() {
        let p = page(&doc(
            "",
            "      - name: list\n        component: collection\n        reads: t.V\n        columns: [title]\n        row_actions:\n          - name: close\n            does: t.Close\n            bind: {id: row.id}\n            label: Close\n            confirm: {title: Close this one?, confirm_label: Close it, show: row.priority == high}\n",
        ));
        assert!(p.contains("const cf_list_close = useConfirm();"), "{p}");
        assert!(
            p.contains("@click=\"same(row['priority'], 'high') ? cf_list_close.ask(row) : c_list_close.run({ id: row['id'] as Value })\""),
            "{p}"
        );
        assert!(
            p.contains("<template v-if=\"cf_list_close.at === row\">"),
            "{p}"
        );
        assert!(p.contains("Close this one?"), "{p}");
        assert!(p.contains(">Close it</KitButton>"), "{p}");
        let named = doc(
            "",
            "      - name: list\n        component: collection\n        reads: t.V\n        columns: [title]\n        row_actions: [{name: close, does: t.Close, label: Close, confirm: close_dialog}]\n",
        );
        assert_refused_at(
            &named,
            "pages/p/sections/list/row_actions/close/confirm",
            "overlay",
        );
    }

    #[test]
    fn a_confirm_without_a_label_confirms_with_delete() {
        let p = page(&doc(
            "    header:\n      actions: [{name: wipe, does: t.Wipe, label: Wipe, confirm: {title: Wipe all?}}]\n",
            "      - {name: list, component: collection, reads: t.V, columns: [title]}\n",
        ));
        assert!(p.contains(">Delete</KitButton>"), "{p}");
        assert!(p.contains("cf_p_wipe.ask(true)"), "{p}");
    }

    #[test]
    fn a_form_field_as_a_choice_is_a_select_of_its_options() {
        let p = page(&doc(
            "",
            "      - name: new\n        component: form\n        does: t.Open\n        fields:\n          - {field: title}\n          - {field: priority, as: choice, choice: {component: choice, options: [high, {value: 2, label: Normal}]}}\n",
        ));
        assert!(
            p.contains("<KitSelect label=\"Priority\" show-label :options=\"[{ value: 'high', label: 'high' }, { value: 2, label: 'Normal' }]\" :model-value=\"f_new.draft['priority']\" @update:model-value=\"f_new.draft['priority'] = $event\" />"),
            "{p}"
        );
        assert_refused_at(
            &doc(
                "",
                "      - name: new\n        component: form\n        does: t.Open\n        fields:\n          - {field: priority, as: choice, choice: {component: choice, options: Priority}}\n",
            ),
            "pages/p/sections/new/fields/priority/choice/options",
            "enum",
        );
    }
}
