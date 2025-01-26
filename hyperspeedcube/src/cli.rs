use std::io::{Read, Write};
use std::sync::Arc;

use eyre::{eyre, Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// TODO: specify log file via CLI

/// Hyperspeedcube command-line interface
///
/// If no subcommand is specified, then the GUI is opened.
#[derive(Debug, clap::Parser)]
#[command(version, args_conflicts_with_subcommands = true)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
    // /// Log file to open in the GUI.
    // #[arg(value_parser)]
    // pub input_file: Option<clio::Input>,
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Subcommand {
    /// Outputs program info and credits in Markdown.
    About,
    /// Outputs JSON information about a puzzle or generator.
    Puzzle {
        /// Puzzle ID (such as `ft_cube:3`) or generator ID (such as `ft_cube`)
        ids: Vec<String>,
    },
    /// Outputs all puzzle and puzzle generator IDs.
    Puzzles {
        /// List only non-generated puzzles.
        #[arg(short, long)]
        puzzles: bool,
        /// List only generators IDs instead.
        #[arg(short, long)]
        generators: bool,
        /// List only generator examples instead.
        #[arg(short, long)]
        examples: bool,

        /// Include experimental puzzles.
        #[arg(short = 'x', long)]
        experimental: bool,

        /// Query expression to search for.
        query: Option<String>,
    },
    /// Outputs all known tags.
    Tags,
    /// Verifies a log file and outputs JSON.
    Verify {
        /// Log file to verify, use '-' for stdin.
        #[arg(value_parser)]
        log_file: clio::Input,

        /// Don't verify that the puzzle was actually solved.
        #[arg(long)]
        skip_simulation: bool,
    },
}

pub(crate) fn exec(subcommand: Subcommand) -> Result<()> {
    match subcommand {
        Subcommand::About => {
            hyperpuzzle::load_global_catalog();
            println!("{}", crate::about_text());
            Ok(())
        }

        Subcommand::Puzzle { ids } => {
            hyperpuzzle::load_global_catalog();
            let catalog = hyperpuzzle::catalog();
            let puzzles = catalog.puzzles();
            let mut requested_puzzles = vec![];
            for puzzle_id in ids {
                if let Some(generator) = puzzles.generators.get(&puzzle_id) {
                    requested_puzzles.push(generator.meta.clone());
                } else {
                    let puzzle = catalog
                        .build_puzzle_spec_blocking(&puzzle_id)
                        .map_err(|e| eyre!("error building puzzle: {e}"))?;
                    requested_puzzles.push(puzzle.meta.clone());
                }
            }
            write_json_output(&requested_puzzles.iter().collect_vec())?;
            Ok(())
        }

        Subcommand::Puzzles {
            puzzles,
            generators,
            examples,

            experimental,

            query,
        } => {
            let all = !puzzles && !generators && !examples;
            hyperpuzzle::load_global_catalog();
            let puzzle_catalog = hyperpuzzle::catalog().puzzles();
            let mut entries = vec![];

            // Filter by type
            if all || puzzles {
                entries.extend(puzzle_catalog.non_generated.values().map(|v| &v.meta));
            }
            if all || generators {
                entries.extend(puzzle_catalog.generators.values().map(|v| &v.meta));
            }
            if all || examples {
                entries.extend(puzzle_catalog.generated_examples.values().map(|v| &v.meta));
            }

            // Filter by experimental
            let entries = entries
                .into_iter()
                .filter(|meta| experimental || !meta.tags.is_experimental());

            // Filter by query
            let ids = if let Some(q) = query {
                let query = crate::gui::Query::from_str(&q);
                entries
                    .filter_map(|entry| query.try_match(entry))
                    .sorted_unstable()
                    .map(|query_match| &query_match.object.id)
                    .collect_vec()
            } else {
                entries
                    .into_iter()
                    .sorted_unstable()
                    .map(|meta| &meta.id)
                    .collect_vec()
            };

            for id in ids {
                println!("{id}");
            }

            Ok(())
        }

        Subcommand::Tags => {
            for tag in hyperpuzzle_core::TAGS.all_tags() {
                println!("{tag}");
            }
            Ok(())
        }

        Subcommand::Verify {
            mut log_file,
            skip_simulation,
        } => {
            hyperpuzzle::load_global_catalog();
            let mut buffer = String::new();
            log_file
                .read_to_string(&mut buffer)
                .context("error reading log file")?;
            let (log_file, _warnings) = hyperpuzzle_log::LogFile::deserialize(&buffer)
                .context("error deserializing log file")?;

            hyperpuzzle::load_global_catalog();
            let catalog = hyperpuzzle::catalog();

            let facts = log_file
                .solves
                .iter()
                .filter_map(|solve| {
                    if !skip_simulation {
                        hyperpuzzle_log::verify::verify(&catalog, solve)
                    } else {
                        hyperpuzzle_log::verify::verify_without_checking_solution(&catalog, solve)
                    }
                })
                .collect_vec();

            write_json_output(&facts)
        }
    }
}

fn write_json_output<T: Serialize>(value: &T) -> Result<()> {
    serde_json::to_writer_pretty(std::io::stdout(), value)
        .context("error writing verification to output")?;
    println!();
    Ok(())
}
