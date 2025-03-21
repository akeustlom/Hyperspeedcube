use std::path::PathBuf;
use std::sync::Arc;

use hyperpuzzle_log::verify::SolveVerification;
use hyperpuzzle_log::{LogFile, Solve};
use hyperpuzzle_view::PuzzleSimulation;
use hyperstats::NewPbs;

use crate::gui::App;
use crate::gui::util::EguiTempValue;

#[derive(Debug, Clone)]
struct SolveCompletePopup {
    solve: Solve,
    puzzle_name: String,
    file_path: PathBuf,
    file_name: String,
    new_pbs: NewPbs,
    verification: SolveVerification,
    saved: bool,
}

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    let solve_complete_popup = EguiTempValue::<Option<SolveCompletePopup>>::new(ui);

    if let Some(Some(mut popup)) = solve_complete_popup.get().filter(|_| {
        app.active_puzzle
            .with_sim(|sim| sim.special_anim().get().is_none())
            .unwrap_or(true)
    }) {
        let r = egui::Modal::new(unique_id!()).show(ui.ctx(), |ui| {
            ui.heading(format!(
                "Yay! You solved the {} in {} twists",
                popup.puzzle_name, popup.verification.solution_stm_count,
            ));

            if let Some(dur) = popup
                .verification
                .blindsolve_duration
                .filter(|_| popup.new_pbs.blind)
            {
                // TODO: prettify this
                ui.label(format!("You set a new blindsolve PB of {dur}"));
            }

            if let Some(dur) = popup
                .verification
                .speedsolve_duration
                .filter(|_| popup.new_pbs.speed)
            {
                ui.label(format!("You set a new speedsolve PB of {dur}"));
            }

            if popup.new_pbs.fmc {
                let stm = popup.verification.solution_stm_count;
                ui.label(format!("You set a new move count PB of {stm} STM"));
            }

            if popup.saved {
                ui.label(format!("Saved to {}", popup.file_name));
            } else if ui.button("Save this solve").clicked() {
                // Save log file
                if let Some(p) = popup.file_path.parent() {
                    std::fs::create_dir_all(p);
                }
                // TODO: handle error
                if let Ok(()) = std::fs::write(
                    &popup.file_path,
                    LogFile {
                        program: Some(crate::PROGRAM.clone()),
                        solves: vec![popup.solve.clone()],
                    }
                    .serialize(),
                ) {
                    popup.saved = true;
                    solve_complete_popup.set(Some(Some(popup.clone())));

                    // Save PBs
                    if popup.new_pbs.any() {
                        app.stats
                            .record_new_pb(&popup.verification, &popup.file_name);
                        hyperstats::save(&app.stats);
                    }
                }
            }

            if ui.button("Close").clicked() {
                solve_complete_popup.set(None);
            }
        });
        if r.should_close() {
            solve_complete_popup.set(None);
        }
    } else {
        app.active_puzzle.with_sim(|sim| {
            if sim.has_been_fully_scrambled() && sim.handle_newly_solved_state() {
                let solve = sim.serialize();
                let verification = hyperpuzzle_log::verify::verify_without_checking_solution(
                    &hyperpuzzle::catalog(),
                    &solve,
                )?;
                let (file_path, file_name) = hyperpaths::solve_autosave_file(
                    &solve.puzzle.id,
                    &verification.time_completed.to_string(),
                    verification.solution_stm_count,
                )
                .ok()?;
                let new_pbs = app.stats.check_new_pb(&verification);

                if new_pbs.first {
                    sim.start_special_anim();
                }

                solve_complete_popup.set(Some(Some(SolveCompletePopup {
                    solve,
                    puzzle_name: sim.puzzle_type().meta.name.clone(),
                    file_path,
                    file_name,
                    new_pbs,
                    verification,
                    saved: false,
                })));
            }
            None::<std::convert::Infallible>
        });
    }
}
