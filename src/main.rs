#![allow(dead_code, unused_imports)]

extern crate piston_window;
extern crate pretty_env_logger;

use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use color::{BLACK, BLUE, RED, WHITE, YELLOW};
use log::{info, warn};
use piston_window::*;
use rand::{seq::SliceRandom, thread_rng};
use types::Color;
use y_game::*;

const WIDTH: f64 = 1920.0;
const HEIGHT: f64 = 1000.0;
const SCALE: f64 = 900.0;

pub const COLORS: [Color; 4] = [BLACK, RED, BLUE, YELLOW];

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Y-game", [WIDTH as u32, HEIGHT as u32])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut glyphs = window.load_font("assets/Lato-Regular.ttf").unwrap();

    // let positions = get_positions("search.txt");
    // let &(reval, beval, red, blue, player, eval) = positions.choose(&mut thread_rng()).unwrap();

    // let mut game = Game::new((red, blue), player == 1);
    let mut game = Game::new((0, 0), true);

    pretty_env_logger::init();

    let r = 10.0f64;
    let r2 = r.powi(2);
    let mut cursor_loc = [0.0; 2];

    let (tx_ai, rx) = run_ai_thread(0);

    tx_ai.send((game.boards(), game.player())).unwrap();

    // let mut ai_eval = Some((reval, beval, eval, 0, 0));
    let mut ai_eval = None;
    while let Some(e) = window.next() {
        if let Ok(ai_eval_) = rx.try_recv() {
            ai_eval = Some(ai_eval_);
            let best = ai_eval_.1;
            info!("Move {} by player 1", best + 1);
            if let Err(err) = game.play(best) {
                warn!("AI ERROR: {err:?}")
            }
        }

        if e.render_args().is_some() {
            window.draw_2d(&e, |c, g, d| {
                clear(WHITE, g);

                text::Text::new_color(BLACK, 40)
                    .draw(
                        match game.state() {
                            GameState::OnGoing => {
                                if game.player() == 1 {
                                    "Red to Move"
                                } else {
                                    "Blue to Move"
                                }
                            }
                            GameState::RedLose => "Blue Won",
                            GameState::RedWin => "Red Won",
                        },
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(100.0, HEIGHT / 2.0),
                        g,
                    )
                    .unwrap();

                // if let Some((reval, beval, eval, best, transpos)) = ai_eval {
                    if let Some((eval, best, transpos)) = ai_eval {
                    text::Text::new_color(BLACK, 20)
                        .draw(
                            &format!("Evaluation: {eval}"),
                            &mut glyphs,
                            &c.draw_state,
                            c.transform.trans(100.0, HEIGHT / 2.0 + 80.0),
                            g,
                        )
                        .unwrap();
                    text::Text::new_color(BLACK, 20)
                        .draw(
                            &format!("Best Move: {best}"),
                            &mut glyphs,
                            &c.draw_state,
                            c.transform.trans(100.0, HEIGHT / 2.0 + 120.0),
                            g,
                        )
                        .unwrap();
                    text::Text::new_color(BLACK, 20)
                        .draw(
                            &format!("Positions Checked: {transpos}"),
                            &mut glyphs,
                            &c.draw_state,
                            c.transform.trans(100.0, HEIGHT / 2.0 + 160.0),
                            g,
                        )
                        .unwrap();
                    // text::Text::new_color(BLACK, 20)
                    //     .draw(
                    //         &format!("Reval: {reval}"),
                    //         &mut glyphs,
                    //         &c.draw_state,
                    //         c.transform.trans(100.0, HEIGHT / 2.0 + 160.0),
                    //         g,
                    //     )
                    //     .unwrap();
                    // text::Text::new_color(BLACK, 20)
                    //     .draw(
                    //         &format!("Beval: {beval}"),
                    //         &mut glyphs,
                    //         &c.draw_state,
                    //         c.transform.trans(100.0, HEIGHT / 2.0 + 160.0),
                    //         g,
                    //     )
                    //     .unwrap();
                }

                glyphs.factory.encoder.flush(d);

                for (i, adjs) in NEIGHBOURS.iter().enumerate() {
                    for &adj in adjs {
                        if adj != 0 {
                            line(
                                BLACK,
                                0.5,
                                [
                                    WIDTH / 2.0 - SCALE / 2.0 + SCALE * COORDINATES[i].0,
                                    HEIGHT / 2.0 + SCALE / 2.0 - SCALE * COORDINATES[i].1,
                                    WIDTH / 2.0 - SCALE / 2.0
                                        + SCALE * COORDINATES[(adj - 1) as usize].0,
                                    HEIGHT / 2.0 + SCALE / 2.0
                                        - SCALE * COORDINATES[(adj - 1) as usize].1,
                                ],
                                c.transform,
                                g,
                            );
                        }
                    }
                }

                for (i, (x, y)) in COORDINATES.iter().enumerate() {
                    let cx = WIDTH / 2.0 - SCALE / 2.0 + SCALE * x;
                    let cy = HEIGHT / 2.0 + SCALE / 2.0 - SCALE * y;
                    let c = c.trans(cx, cy);
                    if game.last_move() == i as u32 {
                        let r = 15.0;
                        circle_arc(YELLOW, r, 0.0, f64::_360(), [0.0; 4], c.transform, g);
                    }

                    circle_arc(
                        COLORS[game.get(i as u32) as usize],
                        r,
                        0.0,
                        f64::_360(),
                        [0.0; 4],
                        c.transform,
                        g,
                    );
                }
            });
        }

        if let Some(pos) = e.mouse_cursor_args() {
            cursor_loc = pos;
        }

        if let Some(button) = e.press_args() {
            if game.player() == 2 && button == Button::Mouse(MouseButton::Left) {
                let [xm, ym] = cursor_loc;
                for (i, (x, y)) in COORDINATES.iter().enumerate() {
                    let cx = WIDTH / 2.0 - SCALE / 2.0 + SCALE * x;
                    let cy = HEIGHT / 2.0 + SCALE / 2.0 - SCALE * y;
                    let d2 = (xm - cx).powi(2) + (ym - cy).powi(2);
                    if d2 < r2 {
                        info!("Move {} by player 2", i + 1);
                        if let Err(err) = game.play(i as u32) {
                            warn!("ERROR: {err:?}")
                        } else {
                            ai_eval = None;
                            tx_ai.send((game.boards(), game.player())).unwrap();
                        }
                        break;
                    }
                }
            }
            if button == Button::Keyboard(Key::Space) {
                // let &(reval, beval, red, blue, player, eval) = positions.choose(&mut thread_rng()).unwrap();
                // game.reset(red, blue, player == 1);
                // ai_eval = Some((reval, beval, eval, 0, 0));
                game.reset(0, 0, true);
                ai_eval = None;
            }
        }
    }
}

fn run_ai_thread(depth: u32) -> (Sender<((u128, u128), u8)>, Receiver<(i32, u32, usize)>) {
    let (tx_ai, rx_ai) = mpsc::channel();
    let (tx, rx) = mpsc::channel();

    let _ = thread::spawn(move || {
        for (boards, player) in rx_ai {
            if player == 1 {
                let mut transpositions = HashMap::new();
                let (eval, best) =
                    Bot::eval_with_transpos(boards, player, depth, &mut transpositions);
                tx.send((eval, best, transpositions.len())).unwrap();
            }
        }
    });
    (tx_ai, rx)
}

fn get_positions(path: &str) -> Vec<(u32, u32, u128, u128, u8, i32)> {
    std::fs::read_to_string(path)
        .unwrap()
        .split('\n')
        .map(|l| {
            let mut it = l.split_ascii_whitespace();
            (
                it.next().unwrap().parse::<u32>().unwrap(),
                it.next().unwrap().parse::<u32>().unwrap(),
                it.next().unwrap().parse::<u128>().unwrap(),
                it.next().unwrap().parse::<u128>().unwrap(),
                it.next().unwrap().parse::<u8>().unwrap(),
                it.next().unwrap().parse::<i32>().unwrap(),
            )
        })
        .collect()
}
