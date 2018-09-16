pub trait GameState<'a>: 'a + Clone + Ord {
    type It: IntoIterator<Item = Self>;

    // computes if the game is ended in favor of player
    fn win(&self, player: i32) -> bool;

    // compute the value in player +1 perspective
    fn value(&self) -> i32;

    // compute posibilities for player `player`
    fn possibilities(&self, player: i32) -> Self::It;

    // exchange the two players in game state
    fn swap(&mut self);

    // computes the symmetries of the state (including itself)
    fn symmetries(&self) -> Self::It;

    // returns a score in favor of player `player` in `player` perspective (higher is better)
    // do not look for score smaller than `alpha`
    // do not look for score bigger than `beta`
    // self is a state in which it's `player` turn to play
    fn negamax(&self, player: i32, depth: i32, mut alpha: i32, beta: i32) -> i32 {
        // two players: +1 and -1

        if depth == 0 || self.win(-player) {
            return player * self.value() * (depth + 1);
        }

        let mut best_value = -std::i32::MAX;

        for state in self.possibilities(player) {
            let value = -state.negamax(-player, depth - 1, -beta, -alpha);

            if value > best_value {
                best_value = value;
            }
            if value > alpha {
                alpha = value;
            }
            if alpha >= beta {
                break;
            }
        }
        best_value
    }

    fn negamax_table(
        &self,
        player: i32,
        depth: i32,
        mut alpha: i32,
        mut beta: i32,
        table: &mut Table<Self>,
    ) -> i32 {
        if depth == 0 || self.win(-player) {
            return player * self.value() * (depth + 1);
        }

        if depth <= 2 {
            return self.negamax(player, depth, alpha, beta);
        }

        if let Some(s) = table.get(self, player, depth, &mut alpha, &mut beta) {
            return s;
        }

        let orig_alpha = alpha;
        let orig_beta = beta;

        let mut best_value = -std::i32::MAX;

        for state in self.possibilities(player) {
            let value = -state.negamax_table(-player, depth - 1, -beta, -alpha, table);

            if value > best_value {
                best_value = value;
            }
            if value > alpha {
                alpha = value;
            }
            if alpha >= beta {
                break;
            }
        }

        table.insert(
            self.clone(),
            player,
            depth,
            orig_alpha,
            orig_beta,
            best_value,
        );
        best_value
    }

    // compute the value in player +1 perspective
    // turn of `player` to play
    fn negamax_value(&self, player: i32, depth: i32, table: &mut Table<Self>) -> i32 {
        player * self.negamax_table(player, depth, -std::i32::MAX, std::i32::MAX, table)
    }

    fn bot_play(&self, player: i32, depth: i32, table: &mut Table<Self>) -> Vec<Self> {
        let mut best_value = -std::i32::MAX;
        let mut results = Vec::new();

        for state in self.possibilities(player) {
            let value = -state.negamax_table(-player, depth, -std::i32::MAX, std::i32::MAX, table);

            if value > best_value {
                best_value = value;
                results.clear();
            }
            if value == best_value {
                results.push(state);
            }
        }

        results
    }
}

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Interval {
    Upperbound(i32),
    Lowerbound(i32),
    Range(i32, i32),
    Exact(i32),
    Unconstrained,
}

use std::cmp::{max, min};

impl std::ops::Add for Interval {
    type Output = Interval;
    fn add(self, rhs: Interval) -> Interval {
        match (self, rhs) {
            (Interval::Unconstrained, y) => y,
            (Interval::Exact(x), _) => Interval::Exact(x),
            (Interval::Upperbound(xb), Interval::Upperbound(yb)) => {
                Interval::Upperbound(min(xb, yb))
            }
            (Interval::Upperbound(xb), Interval::Lowerbound(ya)) => if ya == xb {
                Interval::Exact(ya)
            } else {
                Interval::Range(ya, xb)
            },
            (Interval::Upperbound(xb), Interval::Range(ya, yb)) => if ya == xb {
                Interval::Exact(ya)
            } else {
                Interval::Range(ya, min(xb, yb))
            },
            (Interval::Lowerbound(xa), Interval::Lowerbound(ya)) => {
                Interval::Lowerbound(max(xa, ya))
            }
            (Interval::Lowerbound(xa), Interval::Range(ya, yb)) => if xa == yb {
                Interval::Exact(xa)
            } else {
                Interval::Range(max(xa, ya), yb)
            },
            (Interval::Range(xa, xb), Interval::Range(ya, yb)) => if xa == yb {
                Interval::Exact(xa)
            } else if ya == xb {
                Interval::Exact(ya)
            } else {
                Interval::Range(max(xa, ya), min(xb, yb))
            },
            (x, y) => y + x,
        }
    }
}

#[derive(Clone)]
pub struct Table<S: Ord>(BTreeMap<(i32, S), Interval>);

impl<'a, S> Default for Table<S>
where
    S: GameState<'a>,
{
    fn default() -> Table<S> {
        Table::new()
    }
}

impl<'a, S> Table<S>
where
    S: GameState<'a>,
{
    pub fn new() -> Table<S> {
        Table(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(
        &self,
        state: &S,
        player: i32,
        depth: i32,
        alpha: &mut i32,
        beta: &mut i32,
    ) -> Option<i32> {
        if player == -1 {
            let mut state = state.clone();
            state.swap();
            return self.get(&state, 1, depth, alpha, beta);
        }

        let state = state.symmetries().into_iter().min().unwrap();

        if let Some(&entry) = self.0.get(&(depth, state)) {
            match entry {
                Interval::Exact(value) => {
                    return Some(value);
                }
                Interval::Upperbound(b) => {
                    if b < *beta {
                        *beta = b;
                    }
                }
                Interval::Lowerbound(a) => {
                    if a > *alpha {
                        *alpha = a;
                    }
                }
                Interval::Range(a, b) => {
                    if a > *alpha {
                        *alpha = a;
                    }
                    if b < *beta {
                        *beta = b;
                    }
                }
                Interval::Unconstrained => {}
            }

            if *alpha >= *beta {
                return Some(*alpha);
            }
        }
        None
    }

    pub fn insert(
        &mut self,
        mut state: S,
        player: i32,
        depth: i32,
        alpha: i32,
        beta: i32,
        value: i32,
    ) {
        if player == -1 {
            // allways use the player +1 perspective
            state.swap();
        }

        let state = state.symmetries().into_iter().min().unwrap();
        let key = (depth, state);

        let entry = if value <= alpha {
            Interval::Upperbound(value) // le score de `state` est de au maximum `score`
        } else if beta <= value {
            Interval::Lowerbound(value) // le score de `state` est de au moins `score`
        } else {
            Interval::Exact(value)
        };

        let old = self.0.entry(key).or_insert(Interval::Unconstrained);
        *old = *old + entry;
    }
}
