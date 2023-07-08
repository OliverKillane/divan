use std::str::Split;

use crate::{bench::Context, Bencher};

/// Compile-time benchmark entry generated by `#[divan::bench]`.
pub struct Entry {
    /// The benchmarked function's name.
    pub name: &'static str,

    /// The benchmarked function's `module_path!()`.
    pub module_path: &'static str,

    /// `self.module_path + self.name`.
    pub full_path: &'static str,

    /// The file where the benchmarked function was defined.
    pub file: &'static str,

    /// The line where the benchmarked function was defined.
    pub line: u32,

    /// Whether this entry was marked with [`#[ignore]`](https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute).
    pub ignore: bool,

    /// The benchmarking loop.
    pub bench_loop: BenchLoop,
}

type EntryPathComponents = Split<'static, &'static str>;

impl Entry {
    fn module_components(&self) -> EntryPathComponents {
        self.module_path.split("::")
    }

    pub(crate) fn sorting_key(&self) -> impl Ord {
        // Sort benchmarks by alphabetical order, breaking ties using location.
        (self.full_path, self.file, self.line)
    }
}

/// Entries generated by `#[divan::bench]`.
#[linkme::distributed_slice]
pub static ENTRIES: [Entry] = [..];

/// `Entry` benchmarking loop.
pub enum BenchLoop {
    /// Statically-constructed without context.
    Static(fn(&mut Context)),

    /// Runtime-constructed with context.
    Runtime(fn(Bencher)),
}

/// `Entry` tree organized by path components.
pub enum EntryTree<'a> {
    Leaf(&'a Entry),
    Parent { name: &'a str, children: Vec<Self> },
}

impl<'a> EntryTree<'a> {
    /// Constructs a tree from an iterator of entries in the order they're
    /// produced.
    pub fn from_entries<I>(entries: I) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a Entry>,
    {
        let mut result = Vec::<Self>::new();

        for entry in entries {
            Self::insert(&mut result, entry, &mut entry.module_components());
        }

        result
    }

    /// Returns the maximum span for a name in `tree`.
    pub fn max_name_span(tree: &[Self], depth: usize) -> usize {
        tree.iter()
            .map(|node| {
                let node_name_len = node.name().chars().count();
                let node_name_span = node_name_len + (depth * 4);

                let children_max = Self::max_name_span(node.children(), depth + 1);

                node_name_span.max(children_max)
            })
            .max()
            .unwrap_or_default()
    }

    /// Helper for constructing a tree.
    ///
    /// This uses recursion because the iterative approach runs into limitations
    /// with mutable borrows.
    fn insert(tree: &mut Vec<Self>, entry: &'a Entry, rem_modules: &mut EntryPathComponents) {
        if let Some(current_module) = rem_modules.next() {
            if let Some(children) = Self::get_children(tree, current_module) {
                Self::insert(children, entry, rem_modules);
            } else {
                tree.push(Self::from_path(entry, current_module, rem_modules));
            }
        } else {
            tree.push(Self::Leaf(entry));
        }
    }

    /// Constructs a sequence of branches from a module path.
    fn from_path(
        entry: &'a Entry,
        current_module: &'a str,
        rem_modules: &mut EntryPathComponents,
    ) -> Self {
        let child = if let Some(next_module) = rem_modules.next() {
            Self::from_path(entry, next_module, rem_modules)
        } else {
            Self::Leaf(entry)
        };
        Self::Parent { name: current_module, children: vec![child] }
    }

    /// Finds the `Parent.children` for the corresponding module in `tree`.
    fn get_children<'t>(tree: &'t mut [Self], module: &str) -> Option<&'t mut Vec<Self>> {
        tree.iter_mut().find_map(|tree| match tree {
            Self::Parent { name, children } if *name == module => Some(children),
            _ => None,
        })
    }

    fn name(&self) -> &'a str {
        match self {
            Self::Leaf(entry) => entry.name,
            Self::Parent { name, .. } => name,
        }
    }

    fn children(&self) -> &[Self] {
        match self {
            Self::Leaf { .. } => &[],
            Self::Parent { children, .. } => children,
        }
    }
}
