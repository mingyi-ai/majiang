use mahjong_core::bits::MahjongBitArray;
use mahjong_core::engine::{Engine, EngineInput, PlayerId, SeatingPlan};
use mahjong_core::event::{EngineEvent, Reaction};
mod helper;
use helper::TerminalChoiceExt;
use mahjong_core::round::Wind;
use std::collections::HashMap;
use std::io::{self, Write};

pub type IsBotMap = HashMap<PlayerId, bool>;

fn main() {
    let is_bot_map: IsBotMap =
        HashMap::from([(0, false), (1, true), (2, true), (3, true)]);
    let seating_plan: SeatingPlan = SeatingPlan {
        east: 0,
        south: 1,
        west: 2,
        north: 3,
    };
    let mut engine = Engine::new_human_vs_ai(Wind::East, seating_plan);
    let mut input: Option<EngineInput> = None;
    loop {
        let engine_output = engine.step(input).unwrap();
        input = match engine_output {
            EngineEvent::EngineExited => {
                println!("Game engine has exited.");
                break;
            }
            EngineEvent::RoundStarted { round_wind } => {
                println!("Round started with wind: {:?}", round_wind);
                None
            }
            EngineEvent::RequestAction(self_actions) => {
                if is_bot_map
                    .get(&engine.seating_plan[engine.round.turn])
                    .copied()
                    .unwrap_or(true)
                {
                    // Dummy bot logic: always choose the first option
                    let chosen = self_actions[0];
                    println!(
                        "Bot at {:?} chose action: {:?}",
                        engine.round.turn, chosen
                    );
                    Some(EngineInput::ChosenSelfAction(chosen))
                } else {
                    println!(
                        "Your hand: {:?}",
                        engine
                            .round
                            .seats
                            .get(engine.round.turn)
                            .hand
                            .tiles
                            .to_non_flower_tiles()
                    );
                    let chosen = self_actions
                        .as_slice()
                        .choose_debug("Choose your action:")
                        .expect(
                            "At least one self action should be available",
                        );
                    Some(EngineInput::ChosenSelfAction(chosen))
                }
            }
            EngineEvent::RequestDiscard(discard_options) => {
                if is_bot_map
                    .get(&engine.seating_plan[engine.round.turn])
                    .copied()
                    .unwrap_or(true)
                {
                    // Dummy bot logic: always choose the first option
                    let chosen = discard_options[0];
                    println!(
                        "Bot at {:?} chose discard: {:?}",
                        engine.round.turn, chosen
                    );
                    Some(EngineInput::ChosenDiscard(chosen))
                } else {
                    println!(
                        "Your hand: {:?}",
                        engine
                            .round
                            .seats
                            .get(engine.round.turn)
                            .hand
                            .tiles
                            .to_non_flower_tiles()
                    );
                    let chosen = discard_options
                        .as_slice()
                        .choose_debug("Choose your discard:")
                        .expect(
                            "At least one discard option should be available",
                        );
                    Some(EngineInput::ChosenDiscard(chosen))
                }
            }
            EngineEvent::RequestReaction(reaction_options) => {
                let len = reaction_options.len();

                let mut chosen: Vec<Reaction> = Vec::with_capacity(len);
                for seat in [
                    engine.round.turn.next(),
                    engine.round.turn.next().next(),
                    engine.round.turn.next().next().next(),
                ] {
                    let seat_options = reaction_options
                        .get(&seat)
                        .unwrap_or(&Vec::new())
                        .clone();
                    if seat_options.is_empty() {
                        continue;
                    }

                    if is_bot_map
                        .get(&engine.seating_plan[seat])
                        .copied()
                        .unwrap_or(true)
                    {
                        // Dummy bot logic: always choose the first option
                        chosen.push(seat_options[0]);
                        println!(
                            "Bot at {:?} chose reaction: {:?}",
                            seat, seat_options[0]
                        );
                        continue;
                    }
                    println!(
                        "Hand for {:?}: {:?}",
                        seat,
                        engine
                            .round
                            .seats
                            .get(seat)
                            .hand
                            .tiles
                            .to_non_flower_tiles()
                    );
                    chosen.push(
                        seat_options
                            .as_slice()
                            .choose_debug(&format!(
                                "Choose reaction for {:?} (or skip):",
                                seat
                            ))
                            .unwrap_or(Reaction::Skip { seat }),
                    );
                }
                Some(EngineInput::ChosenReactions(chosen))
            }
            EngineEvent::RoundEnded { .. } => {
                println!("Round ended.");
                println!("Choose to exit or continue? (e/c): ");
                io::stdout().flush().ok();
                let mut line = String::new();
                io::stdin().read_line(&mut line).ok();
                match line.trim() {
                    "e" | "E" => Some(EngineInput::Exit),
                    "c" | "C" => None,
                    _ => {
                        println!("Invalid input, continuing by default.");
                        None
                    }
                }
            }
            _ => {
                println!("Engine Event: {:?}", engine_output);
                None
            }
        }
    }
}
