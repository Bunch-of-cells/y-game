use log::info;

use crate::*;
use std::collections::{HashMap, VecDeque};

const INF: i32 = 10_000;

pub struct Bot;

impl Bot {
    pub fn distance_from_edges(red: u128, blue: u128) -> [[u32; 3]; 93] {
        let mut dist_from_edges = [[INF as u32; 3]; 93];
        for (i, side) in SIDES.iter().enumerate() {
            let mut q = VecDeque::new();
            let mut visited = 0u128;

            for j in 0..93u32 {
                if side & (1 << j) != 0 {
                    dist_from_edges[j as usize][i] = 0;
                    let pl = if red & (1 << j) != 0 {
                        1
                    } else if blue & (1 << j) != 0 {
                        2
                    } else {
                        0
                    };
                    q.push_back((j, pl));
                }
            }

            while !q.is_empty() {
                let (current, pl) = q.pop_front().unwrap();
                let dist = dist_from_edges[current as usize][i];
                let mask = 1 << current;
                visited |= mask;

                let barriers = if pl == 1 {
                    blue
                } else if pl == 2 {
                    red
                } else {
                    0
                };
                let bridges = if pl == 1 {
                    red
                } else if pl == 2 {
                    blue
                } else {
                    0
                };

                for adj in NEIGHBOURS[current as usize] {
                    if adj == 0 {
                        continue;
                    }
                    let adj = adj - 1;
                    let mask = 1 << adj;
                    if (visited & mask != 0) || (barriers & mask != 0) {
                        continue;
                    }

                    if bridges & mask != 0 {
                        if dist < dist_from_edges[adj as usize][i] {
                            dist_from_edges[adj as usize][i] = dist;
                            q.push_front((adj, pl));
                        }
                    } else if dist + 1 < dist_from_edges[adj as usize][i] {
                        dist_from_edges[adj as usize][i] = dist + 1;
                        q.push_back((
                            adj,
                            if pl != 0 {
                                pl
                            } else if red & mask != 0 {
                                1
                            } else if blue & mask != 0 {
                                2
                            } else {
                                0
                            },
                        ));
                    }
                }
            }
        }

        let mut dist_from_corners = [[INF as u32; 3]; 93];

        for (i, corner) in [(0, 0), (1, 8), (2, 16)] {
            let mut q = VecDeque::new();
            let mut visited = 0u128;

            dist_from_corners[corner as usize][i] = 0;
            let pl = if red & (1 << corner) != 0 {
                1
            } else if blue & (1 << corner) != 0 {
                2
            } else {
                0
            };
            q.push_back((corner, pl));

            while !q.is_empty() {
                let (current, pl) = q.pop_front().unwrap();
                let dist = dist_from_corners[current as usize][i];
                let mask = 1 << current;
                visited |= mask;

                let barriers = if pl == 1 {
                    blue
                } else if pl == 2 {
                    red
                } else {
                    0
                };
                let bridges = if pl == 1 {
                    red
                } else if pl == 2 {
                    blue
                } else {
                    0
                };

                for adj in NEIGHBOURS[current as usize] {
                    if adj == 0 {
                        continue;
                    }
                    let adj = adj - 1;
                    let mask = 1 << adj;
                    if (visited & mask != 0) || (barriers & mask != 0) {
                        continue;
                    }

                    if bridges & mask != 0 {
                        if dist < dist_from_corners[adj as usize][i] {
                            dist_from_corners[adj as usize][i] = dist;
                            q.push_front((adj, pl));
                        }
                    } else if dist + 1 < dist_from_corners[adj as usize][i] {
                        dist_from_corners[adj as usize][i] = dist + 1;
                        q.push_back((
                            adj,
                            if pl != 0 {
                                pl
                            } else if red & mask != 0 {
                                1
                            } else if blue & mask != 0 {
                                2
                            } else {
                                0
                            },
                        ));
                    }
                }
            }
        }

        for i in 0..93 {
            let [x, y, z] = &mut dist_from_edges[i];
            let [a, b, c] = dist_from_corners[i];
            let mut d = [
                (0, *x + *y + *z),
                (1, a + *y),
                (2, b + *z),
                (3, c + *x),
                (4, a + b),
                (5, b + c),
                (6, c + a),
            ];
            d.sort_by_key(|&(_, x)| x);
            match d[0] {
                (0, _) => (),
                (1, _) => {
                    *x = a / 2;
                    *z = a / 2;
                }
                (2, _) => {
                    *x = b / 2;
                    *y = b / 2;
                }
                (3, _) => {
                    *y = c / 2;
                    *z = c / 2;
                }
                (4, _) => {
                    *z = a / 2;
                    *y = b / 2;
                    *x = (a + b) / 2;
                }
                (5, _) => {
                    *x = b / 2;
                    *z = c / 2;
                    *y = (c + b) / 2;
                }
                (6, _) => {
                    *x = a / 2;
                    *y = c / 2;
                    *z = (a + c) / 2;
                }
                _ => unreachable!(),
            }
        }

        dist_from_edges
    }

    pub fn all_strings(red: u128, blue: u128) -> Vec<(u32, bool)> {
        let mut outvisited = 0u128;
        let mut dist_of_strings = Vec::new();
        let distances_to_edges = Self::distance_from_edges(red, blue);

        for node in 0..93u32 {
            let mask = 1 << node;
            if (red | blue) & mask == 0 {
                continue;
            }
            if outvisited & mask != 0 {
                continue;
            }

            let mut q = VecDeque::new();
            let mut visited = 0u128;
            let mut dist_to_edges = [INF as u32; 3];

            visited |= 1 << node;

            let is_red = red & mask != 0;
            let bridges = if is_red { red } else { blue };

            q.push_back(node);
            while !q.is_empty() {
                let current = q.pop_front().unwrap();

                for i in 0..3 {
                    if distances_to_edges[current as usize][i] < dist_to_edges[i] {
                        dist_to_edges[i] = distances_to_edges[current as usize][i];
                    }
                }

                for adj in NEIGHBOURS[current as usize] {
                    if adj == 0 {
                        continue;
                    }
                    let adj = adj - 1;
                    let mask = 1 << adj;
                    if (visited & mask == 0) && (bridges & mask != 0) {
                        visited |= mask;
                        q.push_back(adj);
                    }
                }
            }

            outvisited |= visited;
            dist_of_strings.push((dist_to_edges.into_iter().sum(), is_red));
        }
        dist_of_strings
    }

    pub fn static_eval(boards: (u128, u128)) -> i32 {
        let (red, blue) = boards;
        let mut red_cost = INF as u32;
        let mut blue_cost = INF as u32;
        for (cost, is_red) in Self::all_strings(red, blue) {
            if is_red {
                if cost < red_cost {
                    red_cost = cost;
                }
            } else if cost < blue_cost {
                blue_cost = cost;
            }
        }

        // print!("{} {} ", red_cost, blue_cost);

        info!("RED: {red_cost} :: BLUE: {blue_cost}");
        
        if red_cost == 0 {
            INF
        } else if blue_cost == 0 {
            -INF
        } else {
            -(red_cost as i32).pow(2) + (blue_cost as i32).pow(2)
        }
    }

    pub fn minimax(
        game: Game,
        depth: u32,
        maximizing: bool,
        mut alpha: i32,
        mut beta: i32,
        trasnpositions: &mut HashMap<(u128, u128, u8), i32>,
    ) -> (i32, u32) {
        if let Some(&best) = trasnpositions.get(&(game.boards().0, game.boards().1, game.player()))
        {
            return (best, 93);
        }
        match game.state() {
            GameState::RedWin => return (INF, 93),
            GameState::RedLose => return (-INF, 93),
            GameState::OnGoing => (),
        }
        if depth == 0 {
            let eval = Self::static_eval(game.boards());
            // println!(
            //     "{} {} {} {}",
            //     game.boards().0,
            //     game.boards().1,
            //     game.player(),
            //     eval
            // );
            return (eval, 93);
        }

        let board = game.boards().0 | game.boards().1;
        let mut visited = 0u128;
        let mut best_move = 93;

        let mut best = if maximizing { -2 * INF } else { 2 * INF };
        for i in NEIGHBOURS[game.last_move() as usize]
            .into_iter()
            .chain(0..93u32)
        {
            if visited & 1 << i != 0 {
                continue;
            }
            visited |= 1 << i;
            if board & (1 << i) != 0 {
                continue;
            }
            let mut new = game.clone();
            new.play_inner(i);
            new.update_winner();
            let k = (new.boards().0, new.boards().1, new.player());
            let (eval, _) = Self::minimax(new, depth - 1, !maximizing, alpha, beta, trasnpositions);

            if maximizing {
                if eval > best {
                    best = eval;
                    best_move = i;
                }
                if best > alpha {
                    alpha = best;
                }
            } else {
                if eval < best {
                    best = eval;
                    best_move = i;
                }
                if best < beta {
                    beta = best;
                }
            }

            trasnpositions.insert(k, best);
            if beta <= alpha {
                break;
            }
        }
        (best, best_move)
    }

    pub fn eval(boards: (u128, u128), player: u8, depth: u32) -> (i32, u32) {
        Self::eval_with_transpos(boards, player, depth, &mut HashMap::new())
    }

    pub fn eval_with_transpos(
        boards: (u128, u128),
        player: u8,
        depth: u32,
        transpositions: &mut HashMap<(u128, u128, u8), i32>,
    ) -> (i32, u32) {
        if depth == 0 {
            for i in 0..93u32 {
                if (boards.0 | boards.1) & (1 << i) != 0 {
                    continue;
                }
                return (Self::static_eval(boards), i);
            }
            unreachable!()
        }

        let a = Self::minimax(
            Game::new(boards, player == 1),
            depth,
            player == 1,
            -3 * INF,
            3 * INF,
            transpositions,
        );
        info!(
            "Positions evaluated: {}, Eval: {}, Best Move:{}",
            transpositions.len(),
            a.0,
            a.1
        );
        a
    }
}
