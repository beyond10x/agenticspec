//! `agenticspec ui generate --project`: the app of an `ess-ui/1` document as a project that builds
//! on its own with node, from the sources the operator app of `agenticspec ui build` is built
//! from.
//!
//! The kit is not published as a package; the operator app is built in this repository over the
//! sources of `ui/kit/src` with the npm manifest and lock of `ui/`. The project carries the same:
//! the kit's sources under `kit/` byte for byte, a manifest that pins the versions of vue, vite
//! and its vue plugin of `ui/package.json`, and the part of `ui/package-lock.json` those pins
//! install. Its build config has the operator app's output options (`index.html`, `app.js`,
//! `app.css`, one chunk, every asset inlined), and its packer writes the tar the way
//! [`crate::operator::archive`] does, so `npm ci && npm run build` gives `dist/` and `dist.tar`
//! with the sha256 a binding names.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::Files;

/// The sources of the kit, by their path below `ui/kit/src`, sorted.
pub const KIT: &[(&str, &str)] = &[
    ("base.css", include_str!("../../../ui/kit/src/base.css")),
    (
        "components/KitBadge.vue",
        include_str!("../../../ui/kit/src/components/KitBadge.vue"),
    ),
    (
        "components/KitButton.vue",
        include_str!("../../../ui/kit/src/components/KitButton.vue"),
    ),
    (
        "components/KitClamp.vue",
        include_str!("../../../ui/kit/src/components/KitClamp.vue"),
    ),
    (
        "components/KitCodeBox.vue",
        include_str!("../../../ui/kit/src/components/KitCodeBox.vue"),
    ),
    (
        "components/KitCopy.vue",
        include_str!("../../../ui/kit/src/components/KitCopy.vue"),
    ),
    (
        "components/KitEmpty.vue",
        include_str!("../../../ui/kit/src/components/KitEmpty.vue"),
    ),
    (
        "components/KitError.vue",
        include_str!("../../../ui/kit/src/components/KitError.vue"),
    ),
    (
        "components/KitField.vue",
        include_str!("../../../ui/kit/src/components/KitField.vue"),
    ),
    (
        "components/KitForm.vue",
        include_str!("../../../ui/kit/src/components/KitForm.vue"),
    ),
    (
        "components/KitIcon.vue",
        include_str!("../../../ui/kit/src/components/KitIcon.vue"),
    ),
    (
        "components/KitLink.vue",
        include_str!("../../../ui/kit/src/components/KitLink.vue"),
    ),
    (
        "components/KitList.vue",
        include_str!("../../../ui/kit/src/components/KitList.vue"),
    ),
    (
        "components/KitLoading.vue",
        include_str!("../../../ui/kit/src/components/KitLoading.vue"),
    ),
    (
        "components/KitMetric.vue",
        include_str!("../../../ui/kit/src/components/KitMetric.vue"),
    ),
    (
        "components/KitNav.vue",
        include_str!("../../../ui/kit/src/components/KitNav.vue"),
    ),
    (
        "components/KitPage.vue",
        include_str!("../../../ui/kit/src/components/KitPage.vue"),
    ),
    (
        "components/KitPanel.vue",
        include_str!("../../../ui/kit/src/components/KitPanel.vue"),
    ),
    (
        "components/KitRecord.vue",
        include_str!("../../../ui/kit/src/components/KitRecord.vue"),
    ),
    (
        "components/KitSearch.vue",
        include_str!("../../../ui/kit/src/components/KitSearch.vue"),
    ),
    (
        "components/KitSelect.vue",
        include_str!("../../../ui/kit/src/components/KitSelect.vue"),
    ),
    (
        "components/KitShell.vue",
        include_str!("../../../ui/kit/src/components/KitShell.vue"),
    ),
    (
        "components/KitSplit.vue",
        include_str!("../../../ui/kit/src/components/KitSplit.vue"),
    ),
    (
        "components/KitStack.vue",
        include_str!("../../../ui/kit/src/components/KitStack.vue"),
    ),
    (
        "components/KitSwitch.vue",
        include_str!("../../../ui/kit/src/components/KitSwitch.vue"),
    ),
    (
        "components/KitTable.vue",
        include_str!("../../../ui/kit/src/components/KitTable.vue"),
    ),
    (
        "components/KitTextInput.vue",
        include_str!("../../../ui/kit/src/components/KitTextInput.vue"),
    ),
    (
        "components/KitThemeToggle.vue",
        include_str!("../../../ui/kit/src/components/KitThemeToggle.vue"),
    ),
    (
        "components/KitToaster.vue",
        include_str!("../../../ui/kit/src/components/KitToaster.vue"),
    ),
    (
        "components/KitToolbar.vue",
        include_str!("../../../ui/kit/src/components/KitToolbar.vue"),
    ),
    (
        "components/KitValue.vue",
        include_str!("../../../ui/kit/src/components/KitValue.vue"),
    ),
    (
        "components/KitZoomCanvas.vue",
        include_str!("../../../ui/kit/src/components/KitZoomCanvas.vue"),
    ),
    ("format.ts", include_str!("../../../ui/kit/src/format.ts")),
    ("index.ts", include_str!("../../../ui/kit/src/index.ts")),
    ("tokens.css", include_str!("../../../ui/kit/src/tokens.css")),
    ("types.ts", include_str!("../../../ui/kit/src/types.ts")),
    ("zoom.ts", include_str!("../../../ui/kit/src/zoom.ts")),
];

/// The npm manifest the operator app is built with.
const UI_PACKAGE: &str = include_str!("../../../ui/package.json");
/// Its lock.
const UI_LOCK: &str = include_str!("../../../ui/package-lock.json");

/// The name of the project's package.
pub const NAME: &str = "agenticspec-ui-app";
/// The build script: vite into `dist/`, then the packer into `dist.tar`.
pub const BUILD: &str = "vite build --logLevel warn && node pack.mjs dist";
/// The packages the project pins, by the section of `ui/package.json` that names them.
pub const PINS: [(&str, &str); 3] = [
    ("dependencies", "vue"),
    ("devDependencies", "@vitejs/plugin-vue"),
    ("devDependencies", "vite"),
];

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="light dark" />
    <title>UI</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="./src/main.ts"></script>
  </body>
</html>
"#;

const MAIN_TS: &str = r"// The host page of the app `agenticspec ui generate --project` wrote: the kit's tokens and styles,
// and one placeholder per host slot of the document (`HOSTS` of src/generated/hosts.ts) that names
// the slot it stands in for. To draw a slot, pass a component of your own under its name instead.
import '@kit';
import { defineComponent, h, type Component } from 'vue';
import { KitEmpty } from '@kit';
import { createGeneratedApp } from './generated/app';
import { HOSTS, type HostComponents } from './generated/hosts';

const placeholder = (name: string): Component =>
  defineComponent({
    name: `Placeholder${name}`,
    inheritAttrs: false,
    setup: () => () => h(KitEmpty, { title: name, message: 'A host slot this build does not draw.' }),
  });

const hosts = Object.fromEntries(HOSTS.map((name) => [name, placeholder(name)])) as unknown as HostComponents;

createGeneratedApp({ hosts }).mount('#app');
";

const VITE_CONFIG: &str = r"import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// The app `agenticspec ui generate --project` wrote: the pages (src/generated), the host page
// (index.html, src/main.ts) and the kit they draw with (kit/). Built as the operator app of
// `agenticspec ui build` is, into dist/ as three files with fixed names, index.html, app.js and
// app.css: one chunk, no hashes in names and every asset inlined, so the file list never changes.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('kit/index.ts') },
    dedupe: ['vue'],
  },
  define: { 'process.env.NODE_ENV': JSON.stringify('production') },
  build: {
    outDir: here('dist'),
    emptyOutDir: true,
    assetsDir: '',
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    cssCodeSplit: false,
    modulePreload: false,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        entryFileNames: 'app.js',
        chunkFileNames: 'app-[name].js',
        assetFileNames: (info) => (info.names?.some((n) => n.endsWith('.css')) ? 'app.css' : '[name][extname]'),
        codeSplitting: false,
      },
    },
  },
});
";

const PACK_MJS: &str = r"// Packs a built directory (dist/ by default) into <dir>.tar the way `agenticspec ui build --archive`
// packs the operator UI: a ustar archive with the entries sorted by name, each a regular file with
// mode 0644, owner and group 0 with no names, and modification time 0, so the same build gives the
// same bytes. Prints the archive's path and sha256, the digest a binding names for it.
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';

const dir = process.argv[2] ?? 'dist';
const names = [];
const walk = (rel) => {
  for (const entry of readdirSync(rel ? join(dir, rel) : dir)) {
    const name = rel ? `${rel}/${entry}` : entry;
    if (statSync(join(dir, name)).isDirectory()) walk(name);
    else names.push(name);
  }
};
walk('');
names.sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));

const octal = (header, at, length, value) => {
  const digits = length - 1;
  header.write(value.toString(8).padStart(digits, '0').slice(-digits), at, 'ascii');
  header[at + digits] = 0;
};

const parts = [];
for (const name of names) {
  const path = Buffer.from(name);
  if (path.length >= 100) {
    console.error(`pack: ${name}: a name of 100 bytes or more`);
    process.exit(1);
  }
  const bytes = readFileSync(join(dir, name));
  const header = Buffer.alloc(512);
  path.copy(header, 0);
  octal(header, 100, 8, 0o644);
  octal(header, 108, 8, 0);
  octal(header, 116, 8, 0);
  octal(header, 124, 12, bytes.length);
  octal(header, 136, 12, 0);
  header.fill(0x20, 148, 156);
  header[156] = 0x30;
  header.write('ustar\0', 257, 'ascii');
  header.write('00', 263, 'ascii');
  let sum = 0;
  for (const b of header) sum += b;
  octal(header, 148, 7, sum);
  header[155] = 0x20;
  parts.push(header, bytes, Buffer.alloc((512 - (bytes.length % 512)) % 512));
}
parts.push(Buffer.alloc(1024));
const tar = Buffer.concat(parts);
const out = join(dirname(dir), `${basename(dir)}.tar`);
writeFileSync(out, tar);
console.log(`archive: ${out}`);
console.log(`archive_sha256: ${createHash('sha256').update(tar).digest('hex')}`);
";

const README: &str = r#"# The app of an ess-ui/1 document

Written by `agenticspec ui generate --project`. Build it with node (20.19 or later, or 22.12 or
later):

```sh
npm ci && npm run build
```

`dist/` then holds `index.html`, `app.js` and `app.css`, and `dist.tar` the same files packed the
way `agenticspec ui build --archive` packs (sorted entries, mode 0644, owner 0, time 0). The build
prints the archive's sha256 as `archive_sha256`; name the archive in a binding as
`ui.<name>.source: {archive: dist.tar, digest: "sha256:<hex>"}`.

- `src/generated/`: the pages, written from the document. Generate again rather than edit them.
- `index.html`, `src/main.ts`: the host page. Each host slot of the document shows a placeholder
  until you pass a component of your own under its name.
- `kit/`: the components the pages draw with, the kit the operator UI of `agenticspec ui build`
  is built from.
- `package.json`, `package-lock.json`: the versions of vue, vite and its vue plugin the operator
  UI is built with.

Generating again replaces this directory, `node_modules/` and `dist/` included.
"#;

/// One edge of the dependency graph, by the field that declares it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Prod,
    Dev,
    Optional,
    Peer,
    PeerOptional,
}

/// The flags npm records on an entry of a lock: reached only through development edges, only
/// through optional edges, only through edges of either kind, only through peer edges.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Flags {
    dev: bool,
    optional: bool,
    dev_optional: bool,
    peer: bool,
}

/// The entry of the package `name` that the package at `from` (`""` for the root) resolves to:
/// in its own `node_modules`, else in the nearest one above it.
fn resolve(packages: &Map<String, Value>, from: &str, name: &str) -> Option<String> {
    let mut base = from.to_string();
    loop {
        let candidate = if base.is_empty() {
            format!("node_modules/{name}")
        } else {
            format!("{base}/node_modules/{name}")
        };
        if packages.contains_key(&candidate) {
            return Some(candidate);
        }
        if base.is_empty() {
            return None;
        }
        base = base
            .rfind("/node_modules/")
            .map_or_else(String::new, |i| base[..i].to_string());
    }
}

/// The edges out of the entry at `path`; the root's are the project's pins.
fn edges(
    path: &str,
    entry: &Value,
    deps: &Map<String, Value>,
    dev: &Map<String, Value>,
) -> Vec<(String, Kind)> {
    let names = |m: &Map<String, Value>, kind: Kind| -> Vec<(String, Kind)> {
        m.keys().map(|n| (n.clone(), kind)).collect()
    };
    if path.is_empty() {
        return [names(deps, Kind::Prod), names(dev, Kind::Dev)].concat();
    }
    let field = |f: &str| entry[f].as_object().cloned().unwrap_or_default();
    let optional = field("optionalDependencies");
    let mut out: Vec<(String, Kind)> = field("dependencies")
        .keys()
        .filter(|n| !optional.contains_key(*n))
        .map(|n| (n.clone(), Kind::Prod))
        .collect();
    out.extend(names(&optional, Kind::Optional));
    for name in field("peerDependencies").keys() {
        let kind = if entry["peerDependenciesMeta"][name]["optional"] == true {
            Kind::PeerOptional
        } else {
            Kind::Peer
        };
        out.push((name.clone(), kind));
    }
    out
}

/// The lock of a package that pins `deps` and `dev`: the entries of `ui_lock` they install, each
/// copied with its flags computed again for this graph as npm computes them.
///
/// # Errors
///
/// A dependency that is not optional and that `ui_lock` does not hold.
pub fn lock(
    ui_lock: &Value,
    deps: &Map<String, Value>,
    dev: &Map<String, Value>,
) -> Result<Value, String> {
    let packages = ui_lock["packages"]
        .as_object()
        .ok_or("ui/package-lock.json: no packages")?;
    let empty = Value::Null;
    let entry = |p: &str| if p.is_empty() { &empty } else { &packages[p] };
    let mut held: BTreeSet<String> = BTreeSet::new();
    let mut queue = vec![String::new()];
    while let Some(path) = queue.pop() {
        for (name, kind) in edges(&path, entry(&path), deps, dev) {
            if kind == Kind::PeerOptional {
                continue;
            }
            match resolve(packages, &path, &name) {
                Some(to) => {
                    if held.insert(to.clone()) {
                        queue.push(to);
                    }
                }
                None if kind == Kind::Optional => {}
                None => {
                    return Err(format!(
                        "ui/package-lock.json: {name}, which {} needs, is not in it",
                        if path.is_empty() {
                            "the project"
                        } else {
                            &path
                        }
                    ));
                }
            }
        }
    }
    let all = Flags {
        dev: true,
        optional: true,
        dev_optional: true,
        peer: true,
    };
    let mut flags: BTreeMap<String, Flags> = held.iter().map(|p| (p.clone(), all)).collect();
    flags.insert(
        String::new(),
        Flags {
            dev: false,
            optional: false,
            dev_optional: false,
            peer: false,
        },
    );
    let nodes: Vec<String> = flags.keys().cloned().collect();
    loop {
        let mut changed = false;
        for path in &nodes {
            let node = flags[path];
            for (name, kind) in edges(path, entry(path), deps, dev) {
                let Some(to) = resolve(packages, path, &name).filter(|t| held.contains(t)) else {
                    continue;
                };
                let edge_dev = kind == Kind::Dev;
                let edge_optional = matches!(kind, Kind::Optional | Kind::PeerOptional);
                let edge_peer = matches!(kind, Kind::Peer | Kind::PeerOptional);
                let unset_dev_optional = !node.dev_optional
                    && !node.dev
                    && !node.optional
                    && !edge_dev
                    && !edge_optional;
                let unset_dev = unset_dev_optional || (!node.dev && !edge_dev);
                let unset_optional = unset_dev_optional || (!node.optional && !edge_optional);
                let unset_peer = !node.peer && !edge_peer;
                let target = flags.get_mut(&to).expect("every held entry has flags");
                let before = *target;
                target.dev &= !unset_dev;
                target.optional &= !unset_optional;
                target.dev_optional &= !unset_dev_optional;
                target.peer &= !unset_peer;
                changed |= *target != before;
            }
        }
        if !changed {
            break;
        }
    }
    let mut out = Map::new();
    out.insert(
        String::new(),
        json!({"name": NAME, "version": "0.0.0", "dependencies": deps, "devDependencies": dev}),
    );
    for path in &held {
        let mut e = packages[path].as_object().cloned().unwrap_or_default();
        for key in ["dev", "optional", "devOptional", "peer", "extraneous"] {
            e.remove(key);
        }
        let f = flags[path];
        if f.peer {
            e.insert("peer".into(), json!(true));
        }
        if f.dev {
            e.insert("dev".into(), json!(true));
        }
        if f.optional {
            e.insert("optional".into(), json!(true));
        }
        if f.dev_optional && !f.dev && !f.optional {
            e.insert("devOptional".into(), json!(true));
        }
        out.insert(path.clone(), Value::Object(e));
    }
    Ok(json!({
        "name": NAME,
        "version": "0.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": out,
    }))
}

fn pretty(v: &Value) -> String {
    let mut text = serde_json::to_string_pretty(v).unwrap_or_default();
    text.push('\n');
    text
}

/// The members of `m` as lines of a JSON object indented by four.
fn members(m: &Map<String, Value>) -> String {
    m.iter()
        .map(|(k, v)| format!("    {}: {v}", Value::String(k.clone())))
        .collect::<Vec<_>>()
        .join(",\n")
}

/// Every file of the project around `app`, the files `crate::generate` wrote, by their path below
/// the project's directory.
///
/// # Errors
///
/// A pin `ui/package.json` does not name, or a package its lock does not hold.
pub fn project(app: &Files) -> Result<Files, String> {
    let ui: Value =
        serde_json::from_str(UI_PACKAGE).map_err(|e| format!("ui/package.json: {e}"))?;
    let ui_lock: Value =
        serde_json::from_str(UI_LOCK).map_err(|e| format!("ui/package-lock.json: {e}"))?;
    let (mut deps, mut dev) = (Map::new(), Map::new());
    for (section, name) in PINS {
        let version = ui[section][name]
            .as_str()
            .ok_or_else(|| format!("ui/package.json: no {section}.{name}"))?;
        let into = if section == "dependencies" {
            &mut deps
        } else {
            &mut dev
        };
        into.insert(name.to_string(), json!(version));
    }
    let package = format!(
        "{{\n  \"name\": \"{NAME}\",\n  \"private\": true,\n  \"version\": \"0.0.0\",\n  \"type\": \"module\",\n  \"description\": \"The app of an ess-ui/1 document, written by agenticspec ui generate --project, over the kit and the versions the operator UI of agenticspec ui build is built with.\",\n  \"scripts\": {{\n    \"build\": {}\n  }},\n  \"dependencies\": {{\n{}\n  }},\n  \"devDependencies\": {{\n{}\n  }}\n}}\n",
        Value::String(BUILD.to_string()),
        members(&deps),
        members(&dev),
    );
    let mut files = Files::new();
    for (name, text) in app {
        files.insert(format!("src/generated/{name}"), text.clone());
    }
    for (name, text) in KIT {
        files.insert(format!("kit/{name}"), (*text).to_string());
    }
    files.insert("README.md".into(), README.into());
    files.insert("index.html".into(), INDEX_HTML.into());
    files.insert("pack.mjs".into(), PACK_MJS.into());
    files.insert("package.json".into(), package);
    files.insert(
        "package-lock.json".into(),
        pretty(&lock(&ui_lock, &deps, &dev)?),
    );
    files.insert("src/main.ts".into(), MAIN_TS.into());
    files.insert("vite.config.ts".into(), VITE_CONFIG.into());
    Ok(files)
}
