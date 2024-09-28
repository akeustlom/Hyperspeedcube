use std::collections::HashMap;
use std::sync::Arc;

use crate::lua::{PuzzleGeneratorSpec, PuzzleSpec};
use crate::Puzzle;

/// Puzzle defined in a Lua file that is cached when constructed.
#[derive(Debug)]
pub(crate) struct LazyPuzzle {
    /// Parameters to construct the puzzle.
    pub spec: Arc<PuzzleSpec>,
    /// Cached constructed puzzle.
    pub constructed: Option<Arc<Puzzle>>,
}
impl LazyPuzzle {
    /// Returns a new lazy puzzle that has not yet been constructed.
    pub fn new(spec: PuzzleSpec) -> Self {
        Self {
            spec: Arc::new(spec),
            constructed: None,
        }
    }
}

/// Puzzle generator defined in a Lua file whose puzzles are cached whenever
/// they are constructed.
#[derive(Debug)]
pub(crate) struct LazyPuzzleGenerator {
    /// Generator to construct a puzzle.
    pub generator: Arc<PuzzleGeneratorSpec>,
    /// Cached constructed puzzles.
    pub constructed: HashMap<String, Arc<Puzzle>>,
}
impl LazyPuzzleGenerator {
    /// Returns a new lazy puzzle generator that has not yet been constructed.
    pub fn new(generator: PuzzleGeneratorSpec) -> Self {
        Self {
            generator: Arc::new(generator),
            constructed: HashMap::new(),
        }
    }
}
