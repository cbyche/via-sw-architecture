//! The real crate graph, read from `cargo metadata`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

use cargo_metadata::{DependencyKind, MetadataCommand};

/// Why the crate graph could not be read.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// `cargo metadata` did not produce a graph. Almost always a manifest that
    /// does not parse, or a workspace member directory that does not exist.
    #[error("`cargo metadata` failed for the workspace at {root}")]
    Metadata {
        /// The workspace root the command was run against.
        root: String,
        /// The underlying cargo failure, including cargo's own stderr.
        #[source]
        source: Box<cargo_metadata::Error>,
    },
}

/// The kind of `Cargo.toml` table a dependency edge was declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeKind {
    /// `[dependencies]` — ships in the library.
    Normal,
    /// `[build-dependencies]` — runs in `build.rs`.
    Build,
    /// `[dev-dependencies]` — tests, benches and examples only.
    Dev,
}

impl EdgeKind {
    /// How the edge is named in a failure message.
    pub const fn label(self) -> &'static str {
        match self {
            EdgeKind::Normal => "dependency",
            EdgeKind::Build => "build-dependency",
            EdgeKind::Dev => "dev-dependency",
        }
    }

    /// Whether the edge is part of what ships.
    ///
    /// Dev edges are excluded from reachability and cycle questions: a
    /// dev-dependency back-edge is legal in cargo and is a normal Rust testing
    /// pattern, so treating it as a path would produce answers about a graph
    /// that never exists at runtime. The band rule still applies to dev edges —
    /// see [`crate::band::Band::allows`] and `tests/crate_graph.rs`.
    pub const fn is_production(self) -> bool {
        matches!(self, EdgeKind::Normal | EdgeKind::Build)
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// One declared dependency from one workspace member to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// The depending package's name.
    pub from: String,
    /// The depended-on package's name.
    pub to: String,
    /// Which dependency table declared it.
    pub kind: EdgeKind,
    /// Whether the dependency is behind a feature (`optional = true`). An
    /// optional dependency is still an edge: the crate may name it.
    pub optional: bool,
    /// The `cfg(...)` the dependency is gated on, if any. Target-gated edges
    /// count — a rule that holds only on macOS is not a rule.
    pub target: Option<String>,
    /// Workspace-relative path of the manifest that declares the edge, so the
    /// failure message can point at the file to open.
    pub manifest: String,
}

impl Edge {
    /// An edge with no feature gate and no target gate.
    ///
    /// For describing a graph by hand — the tests of this crate's own rules,
    /// and any question asked of a graph cargo would refuse to resolve.
    /// `manifest` is set to the conventional location of the depending crate's
    /// manifest; [`CrateGraph::load_from`] reports the real one.
    pub fn new(from: impl Into<String>, to: impl Into<String>, kind: EdgeKind) -> Self {
        let from = from.into();
        let manifest = format!("crates/{from}/Cargo.toml");
        Self {
            from,
            to: to.into(),
            kind,
            optional: false,
            target: None,
            manifest,
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

/// The workspace's own crates and the edges between them.
///
/// Only workspace members are nodes. Third-party dependencies are not part of
/// the layer question and are deliberately absent, which is also why the graph
/// is read with `--no-deps`: no lockfile, no registry, no network.
#[derive(Debug, Clone)]
pub struct CrateGraph {
    root: PathBuf,
    packages: Vec<String>,
    edges: Vec<Edge>,
}

impl CrateGraph {
    /// Read the workspace at [`workspace_root`].
    pub fn load() -> Result<Self, LoadError> {
        Self::load_from(&workspace_root())
    }

    /// Build a graph from an explicit node and edge list.
    ///
    /// Two uses, both real. It is how this crate's own rules are tested — a
    /// gate whose machinery is never shown to catch anything is a gate nobody
    /// should trust. And it is the only way to ask a layer question of a graph
    /// cargo cannot produce: the resolver rejects a production dependency
    /// cycle outright, so [`CrateGraph::production_cycles`] would otherwise be
    /// unreachable code.
    ///
    /// Nodes are sorted and edges ordered by `(from, to, kind)`, matching
    /// [`CrateGraph::load_from`], so message output is stable either way.
    pub fn from_parts(
        root: impl Into<PathBuf>,
        mut packages: Vec<String>,
        mut edges: Vec<Edge>,
    ) -> Self {
        packages.sort();
        packages.dedup();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, left.kind).cmp(&(&right.from, &right.to, right.kind))
        });
        Self {
            root: root.into(),
            packages,
            edges,
        }
    }

    /// Read the workspace rooted at `root`.
    pub fn load_from(root: &Path) -> Result<Self, LoadError> {
        let manifest = root.join("Cargo.toml");
        let metadata = MetadataCommand::new()
            .manifest_path(&manifest)
            .no_deps()
            .exec()
            .map_err(|source| LoadError::Metadata {
                root: root.display().to_string(),
                source: Box::new(source),
            })?;

        let members: BTreeSet<&str> = metadata
            .packages
            .iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .map(|package| package.name.as_ref())
            .collect();

        let workspace_root = metadata.workspace_root.as_std_path();
        let mut edges = Vec::new();
        for package in &metadata.packages {
            if !members.contains(package.name.as_ref()) {
                continue;
            }
            let manifest = package
                .manifest_path
                .as_std_path()
                .strip_prefix(workspace_root)
                .unwrap_or(package.manifest_path.as_std_path())
                .display()
                .to_string();

            for dependency in &package.dependencies {
                // `dependency.name` is the real package name; `rename` is the
                // local alias for `foo = { package = "bar" }`. Matching on the
                // package name is what makes a renamed dependency visible.
                if !members.contains(dependency.name.as_str()) {
                    continue;
                }
                let kind = match dependency.kind {
                    DependencyKind::Normal => EdgeKind::Normal,
                    DependencyKind::Build => EdgeKind::Build,
                    DependencyKind::Development => EdgeKind::Dev,
                    // `DependencyKind` is `#[non_exhaustive]` in spirit: it
                    // carries an `Unknown` variant for kinds a future cargo
                    // might add. Treat an unrecognised kind as a shipping
                    // edge — the strict reading, so a new kind cannot smuggle
                    // a dependency past the band rule.
                    _ => EdgeKind::Normal,
                };
                edges.push(Edge {
                    from: package.name.as_ref().to_owned(),
                    to: dependency.name.clone(),
                    kind,
                    optional: dependency.optional,
                    target: dependency.target.as_ref().map(|target| target.to_string()),
                    manifest: manifest.clone(),
                });
            }
        }

        let mut packages: Vec<String> = members.iter().map(|name| (*name).to_owned()).collect();
        packages.sort();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, left.kind).cmp(&(&right.from, &right.to, right.kind))
        });

        Ok(Self {
            root: workspace_root.to_path_buf(),
            packages,
            edges,
        })
    }

    /// The workspace root cargo reported.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every workspace member, sorted.
    pub fn packages(&self) -> &[String] {
        &self.packages
    }

    /// Every edge between workspace members, sorted by `(from, to, kind)`.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// The edges leaving `package`, of any kind.
    pub fn edges_from<'a>(&'a self, package: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |edge| edge.from == package)
    }

    /// The shortest production path from `from` to `to`, if one exists.
    ///
    /// Production means [`EdgeKind::is_production`]. The path includes both
    /// endpoints, so a direct edge comes back as two entries.
    pub fn production_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for edge in self.edges.iter().filter(|edge| edge.kind.is_production()) {
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .insert(edge.to.as_str());
        }

        let mut came_from: BTreeMap<&str, &str> = BTreeMap::new();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        seen.insert(from);
        queue.push_back(from);

        while let Some(node) = queue.pop_front() {
            if node == to && node != from {
                let mut path = vec![node];
                let mut cursor = node;
                while let Some(previous) = came_from.get(cursor) {
                    path.push(previous);
                    cursor = previous;
                }
                path.reverse();
                return Some(path.into_iter().map(str::to_owned).collect());
            }
            let Some(next) = adjacency.get(node) else {
                continue;
            };
            for candidate in next {
                if seen.insert(candidate) {
                    came_from.insert(candidate, node);
                    queue.push_back(candidate);
                }
            }
        }
        None
    }

    /// Every production dependency cycle, each returned as the loop with its
    /// first node repeated at the end (`a -> b -> a`).
    ///
    /// Dev edges are excluded; see [`EdgeKind::is_production`].
    pub fn production_cycles(&self) -> Vec<Vec<String>> {
        let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for package in &self.packages {
            adjacency.entry(package.clone()).or_default();
        }
        for edge in self.edges.iter().filter(|edge| edge.kind.is_production()) {
            adjacency
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
        }

        let mut marks: BTreeMap<String, Mark> = BTreeMap::new();
        let mut stack: Vec<String> = Vec::new();
        let mut found: Vec<Vec<String>> = Vec::new();
        for package in &self.packages {
            walk(package, &adjacency, &mut marks, &mut stack, &mut found);
        }

        // The same loop can be reached from several entry points; key on the
        // set of nodes so it is reported once.
        let mut seen: BTreeSet<BTreeSet<String>> = BTreeSet::new();
        found.retain(|cycle| seen.insert(cycle.iter().cloned().collect()));
        found
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Active,
    Done,
}

fn walk(
    node: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    marks: &mut BTreeMap<String, Mark>,
    stack: &mut Vec<String>,
    found: &mut Vec<Vec<String>>,
) {
    match marks.get(node) {
        Some(Mark::Done) => return,
        Some(Mark::Active) => {
            if let Some(start) = stack.iter().position(|entry| entry == node) {
                let mut cycle: Vec<String> = stack[start..].to_vec();
                cycle.push(node.to_owned());
                found.push(cycle);
            }
            return;
        }
        None => {}
    }

    marks.insert(node.to_owned(), Mark::Active);
    stack.push(node.to_owned());
    if let Some(targets) = adjacency.get(node) {
        for target in targets {
            walk(target, adjacency, marks, stack, found);
        }
    }
    stack.pop();
    marks.insert(node.to_owned(), Mark::Done);
}

/// The workspace root, resolved from this crate's own location.
///
/// `CARGO_MANIFEST_DIR` is `<root>/crates/via-arch-test`, so the root is two
/// levels up. Deliberately not a hardcoded path and not the current directory:
/// the gate has to give the same answer from a git worktree, a CI checkout and
/// `cargo test` run from any subdirectory.
pub fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(Path::parent) {
        Some(root) => root.to_path_buf(),
        // Unreachable in practice: CARGO_MANIFEST_DIR is always absolute and
        // always at least two components deep for a crate under `crates/`.
        // Falling back to the manifest directory keeps this total rather than
        // panicking, and cargo metadata then fails with a readable error.
        None => manifest_dir.to_path_buf(),
    }
}
