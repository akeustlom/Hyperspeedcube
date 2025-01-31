use std::io::Write;

use eyre::Result;
use hyperpuzzle::PuzzleLintOutput;

use super::{load_puzzle_library, time_it};

/// Whether to lint experimental puzzles. (Non-experimental puzzles are always
/// linted.)
const LINT_EXPERIMENTAL: bool = false;

#[test]
fn lint_all_puzzle_definitions() -> Result<(), String> {
    let lib = load_puzzle_library();

    let mut fail_count = 0;

    let mut out = String::new();

    for puzzle in lib.puzzles() {
        if !LINT_EXPERIMENTAL && puzzle.tags.is_experimental() {
            continue;
        }

        let puzzle_lint_output = time_it(format!("Linting puzzle {}", puzzle.id), || {
            PuzzleLintOutput::from_spec(puzzle)
        })
        .0;

        if !puzzle_lint_output.all_good() {
            fail_count += 1;

            out += &format!("Puzzle {} has lint errors:\n", puzzle_lint_output.puzzle.id);

            let PuzzleLintOutput {
                puzzle: _,
                missing_tags,
            } = puzzle_lint_output;

            if !missing_tags.is_empty() {
                out += "  Missing tags:\n";
                for tag in missing_tags {
                    out += &format!("    {tag:?}\n")
                }
            }
        }
    }

    if fail_count == 0 {
        Ok(())
    } else {
        std::fs::File::create("../lint_output.txt")
            .unwrap()
            .write(out.as_bytes())
            .unwrap();

        Err(format!("{fail_count} puzzles have lint errors"))
    }
}

#[test]
fn build_all_puzzles() -> Result<(), String> {
    let lib = load_puzzle_library();
    let mut failed = vec![];
    let mut times = vec![];
    for puzzle in lib.puzzles() {
        if puzzle.tags.0.contains_key("big") {
            println!(
                "Skipping big puzzle {} ({})",
                puzzle.display_name(),
                puzzle.id
            );
            continue;
        }

        let (result, time) = time_it(
            format!("Building puzzle {} ({})", puzzle.display_name(), puzzle.id),
            || lib.build_puzzle_blocking(&puzzle.id),
        );
        match result {
            Ok(_) => {
                times.push((time, puzzle.display_name().to_owned()));
            }
            Err(_) => {
                println!("Error building {}!", puzzle.display_name());
                failed.push(puzzle);
            }
        }
    }

    times.sort();
    println!();
    println!("Sorted:");
    for (time, puzzle) in times {
        println!("  {time:<11?} {puzzle}");
    }

    if failed.is_empty() {
        Ok(())
    } else {
        let fail_count = failed.len();
        println!();
        println!("{fail_count} puzzles failed to build:");
        for puzzle in failed {
            println!("  {} ({})", puzzle.display_name(), puzzle.id);
        }
        Err(format!("{fail_count} puzzles failed to build:"))
    }
}

#[test]
fn build_7x7x7x7() {
    let lib = load_puzzle_library();
    let (result, time) = time_it("Building puzzle 7x7x7x7", || {
        lib.build_puzzle_blocking("ft_hypercube:7")
    });
    result.expect("failed to build puzzle");
    println!("Done in {time:?}");
}
