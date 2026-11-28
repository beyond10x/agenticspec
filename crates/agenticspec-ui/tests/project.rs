//! `agenticspec ui generate --project`: a project that builds on its own with node, from the
//! same kit sources and the same npm pins the operator app of `agenticspec ui build` is built
//! from, packed the way `ui build --archive` packs. No npm runs here; the files are held to the
//! sources they copy.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use agenticspec_ui::{generate, generate_project};
use serde_json::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn json(name: &str, text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// A derived document with host slots.
const DOC: &str = "ui/views/examples/workstation.ui.yaml";

fn project() -> BTreeMap<String, String> {
    generate_project(&read(DOC)).unwrap_or_else(|r| panic!("{DOC} refused: {r:?}"))
}

fn kit_on_disk() -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let path = e.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path.strip_prefix(base).unwrap().to_string_lossy();
                out.insert(
                    rel.replace('\\', "/"),
                    std::fs::read_to_string(&path).unwrap(),
                );
            }
        }
    }
    let base = repo().join("ui/kit/src");
    let mut out = BTreeMap::new();
    walk(&base, &base, &mut out);
    out
}

/// The project: the generated app under `src/generated/`, the kit under `kit/` byte for byte as
/// `ui/kit/src` holds it, and the host page, build config, manifest, lock and packer beside them.
#[test]
fn the_project_holds_the_generated_app_the_kit_and_a_host_page() {
    let files = project();
    let app = generate(&read(DOC)).unwrap();
    for (name, text) in &app {
        assert_eq!(
            files.get(&format!("src/generated/{name}")),
            Some(text),
            "src/generated/{name}"
        );
    }
    let kit = kit_on_disk();
    let copied: BTreeMap<String, String> = files
        .iter()
        .filter_map(|(n, t)| n.strip_prefix("kit/").map(|r| (r.to_string(), t.clone())))
        .collect();
    assert_eq!(
        copied.keys().collect::<Vec<_>>(),
        kit.keys().collect::<Vec<_>>(),
        "kit/ lists the files of ui/kit/src"
    );
    for (name, text) in &kit {
        assert_eq!(
            &copied[name], text,
            "kit/{name} differs from ui/kit/src/{name}"
        );
    }
    let rest: Vec<&str> = files
        .keys()
        .map(String::as_str)
        .filter(|n| !n.starts_with("kit/") && !n.starts_with("src/generated/"))
        .collect();
    assert_eq!(
        rest,
        [
            "README.md",
            "index.html",
            "pack.mjs",
            "package-lock.json",
            "package.json",
            "src/main.ts",
            "vite.config.ts",
        ]
    );
    let page = &files["index.html"];
    assert!(page.contains("<div id=\"app\"></div>"), "{page}");
    assert!(page.contains("src=\"./src/main.ts\""), "{page}");
    let main = &files["src/main.ts"];
    for needle in [
        "import '@kit';",
        "from './generated/app'",
        "HOSTS",
        ".mount('#app')",
    ] {
        assert!(
            main.contains(needle),
            "src/main.ts lacks {needle:?}:\n{main}"
        );
    }
    assert!(
        files["README.md"].contains("npm ci && npm run build"),
        "{}",
        files["README.md"]
    );
    assert_eq!(project(), files, "the same document gives the same bytes");
}

/// The manifest pins exactly the versions of `ui/package.json` the operator app is built with,
/// and its build script builds with vite and packs `dist/` into `dist.tar`.
#[test]
fn the_manifest_pins_the_versions_the_operator_app_is_built_with() {
    let files = project();
    let pkg = json("package.json", &files["package.json"]);
    let ui = json("ui/package.json", &read("ui/package.json"));
    let pins = [
        ("dependencies", "vue"),
        ("devDependencies", "@vitejs/plugin-vue"),
        ("devDependencies", "vite"),
    ];
    for (section, name) in pins {
        assert!(
            ui[section][name].is_string(),
            "ui/package.json {section}.{name}"
        );
        assert_eq!(pkg[section][name], ui[section][name], "{section}.{name}");
    }
    let named: usize = ["dependencies", "devDependencies"]
        .iter()
        .map(|s| pkg[*s].as_object().map_or(0, serde_json::Map::len))
        .sum();
    assert_eq!(named, pins.len(), "{pkg}");
    assert_eq!(
        pkg["scripts"]["build"],
        "vite build --logLevel warn && node pack.mjs dist"
    );
}

/// The lock is the part of `ui/package-lock.json` those pins install: each entry the same
/// version, tarball and integrity, every dependency it needs inside it, and nothing of the
/// repository's own test tools.
#[test]
fn the_lock_is_the_part_of_the_ui_lock_the_pins_install() {
    let files = project();
    let pkg = json("package.json", &files["package.json"]);
    let lock = json("package-lock.json", &files["package-lock.json"]);
    let ui = json("ui/package-lock.json", &read("ui/package-lock.json"));
    assert_eq!(lock["lockfileVersion"], 3);
    assert_eq!(lock["name"], pkg["name"]);
    let packages = lock["packages"].as_object().unwrap();
    let root = &packages[""];
    assert_eq!(root["name"], pkg["name"]);
    assert_eq!(root["dependencies"], pkg["dependencies"]);
    assert_eq!(root["devDependencies"], pkg["devDependencies"]);
    for (path, entry) in packages.iter().filter(|(p, _)| !p.is_empty()) {
        let from = &ui["packages"][path];
        for key in ["version", "resolved", "integrity"] {
            assert!(entry[key].is_string(), "{path}.{key}");
            assert_eq!(entry[key], from[key], "{path}.{key}");
        }
        let needs = ["dependencies", "peerDependencies"]
            .iter()
            .filter_map(|s| entry[*s].as_object())
            .flat_map(|m| m.keys())
            .filter(|n| entry["peerDependenciesMeta"][n.as_str()]["optional"] != true);
        for name in needs {
            // Node's resolution: the entry's own node_modules, then each one above it.
            let mut base = path.as_str();
            let found = loop {
                if packages.contains_key(&format!("{base}/node_modules/{name}")) {
                    break true;
                }
                match base.rfind("/node_modules/") {
                    Some(i) => base = &base[..i],
                    None => break packages.contains_key(&format!("node_modules/{name}")),
                }
            };
            assert!(found, "{path} needs {name}, which the lock lacks");
        }
    }
    for path in [
        "node_modules/vue",
        "node_modules/@vue/compiler-sfc",
        "node_modules/vite",
        "node_modules/rolldown",
        "node_modules/@vitejs/plugin-vue",
        "node_modules/@rolldown/binding-linux-x64-gnu",
    ] {
        assert!(packages.contains_key(path), "{path}");
    }
    for path in [
        "node_modules/vitest",
        "node_modules/@playwright/test",
        "node_modules/react",
        "node_modules/happy-dom",
        "node_modules/vue-tsc",
    ] {
        assert!(!packages.contains_key(path), "{path}");
    }
    let flag = |p: &str, f: &str| packages[p][f] == true;
    assert!(!flag("node_modules/vue", "dev"));
    assert!(flag("node_modules/vite", "dev"));
    let binding = "node_modules/@rolldown/binding-linux-x64-gnu";
    assert!(flag(binding, "dev") && flag(binding, "optional"));
}

/// The line of `text` starting with `from` and, when it opens a block, every line to the one
/// indented as it was that closes it.
fn block<'a>(text: &'a str, from: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.trim_start().starts_with(from))
        .unwrap_or_else(|| panic!("no line starting with {from:?} in\n{text}"));
    if !lines[start].trim_end().ends_with('{') {
        return vec![lines[start]];
    }
    let indent = lines[start].len() - lines[start].trim_start().len();
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.len() - l.trim_start().len() == indent && l.trim_start().starts_with('}'))
        .map_or(lines.len(), |i| start + 1 + i);
    lines[start..=end].to_vec()
}

/// The build config is the operator app's: the same output options (index.html, app.js and
/// app.css, one chunk, every asset inlined), so `dist/` holds the file names `ui build` writes
/// for its app; the kit is the copy beside it and the output is `dist/`.
#[test]
fn the_build_config_is_the_operator_apps() {
    let files = project();
    let ours = &files["vite.config.ts"];
    let operator = read("ui/operator/vite.config.ts");
    for from in [
        "define:",
        "emptyOutDir:",
        "assetsDir:",
        "assetsInlineLimit:",
        "cssCodeSplit:",
        "modulePreload:",
        "reportCompressedSize:",
        "chunkSizeWarningLimit:",
        "rollupOptions:",
    ] {
        assert_eq!(block(ours, from), block(&operator, from), "{from}");
    }
    for needle in [
        "alias: { '@kit': here('kit/index.ts') }",
        "outDir: here('dist')",
        "base: './'",
        "dedupe: ['vue']",
    ] {
        assert!(
            ours.contains(needle),
            "vite.config.ts lacks {needle:?}:\n{ours}"
        );
    }
}
