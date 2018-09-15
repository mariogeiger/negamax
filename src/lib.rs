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

        for child in self.possibilities(player) {
            let v = -child.negamax(-player, depth - 1, -beta, -alpha);

            if v > best_value {
                best_value = v;
            }
            if v > alpha {
                alpha = v;
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

        for child in self.possibilities(player) {
            let v = -child.negamax_table(-player, depth - 1, -beta, -alpha, table);

            if v > best_value {
                best_value = v;
            }
            if v > alpha {
                alpha = v;
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

        table.clean();
        results
    }
}

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Quality {
    Upperbound,
    Lowerbound,
    Exact,
}

#[derive(Clone, Copy)]
struct TableEntry {
    value: i32,
    depth: i32,
    quality: Quality,
}

#[derive(Clone)]
pub struct Table<S: Ord>(BTreeMap<S, Vec<TableEntry>>);

impl<'a, S> Table<S>
where
    S: GameState<'a>,
{
    pub fn new() -> Table<S> {
        Table(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        let mut x = 0;
        for (_, list) in self.0.iter() {
            x += list.len();
        }
        x
    }

    pub fn get(
        &self,
        state: &S,
        player: i32,
        depth: i32,
        alpha: &mut i32,
        beta: &mut i32,
    ) -> Option<i32> {
        if player == 1 {
            let mut cpy: S = state.clone();
            cpy.swap();

            return self.get(&cpy, 0, depth, alpha, beta);
        }

        if let Some(vs) = self.0.get(state) {
            for entry in vs.iter() {
                if entry.depth == depth {
                    match entry.quality {
                        Quality::Exact => {
                            return Some(entry.value);
                        }
                        Quality::Upperbound => {
                            if entry.value < *beta {
                                *beta = entry.value;
                            }
                        }
                        Quality::Lowerbound => {
                            if entry.value > *alpha {
                                *alpha = entry.value;
                            }
                        }
                    }

                    if *alpha >= *beta {
                        return Some(entry.value);
                    }
                }
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
        score: i32,
    ) {
        if player == -1 {
            // allways use the player +1 perspective
            state.swap();
        }

        let entry = TableEntry {
            value: score,
            depth: depth,
            quality: if score <= alpha {
                Quality::Upperbound // le score de `state` est de au maximum `score`
            } else if beta <= score {
                Quality::Lowerbound // le score de `state` est de au moins `score`
            } else {
                Quality::Exact
            },
        };

        for s in state.symmetries() {
            if let Some(vs) = self.0.get_mut(&s) {
                vs.push(entry);
                continue;
            }
            self.0.insert(s.clone(), vec![entry]);
        }
    }

    // remove useless entries
    pub fn clean(&mut self) {
        for (_, list) in self.0.iter_mut() {
            let mut i = 0;
            'iloop: while i < list.len() {
                for j in 0..list.len() {
                    if i != j
                        && list[j].depth >= list[i].depth
                        && (list[j].quality == Quality::Exact || list[j].quality == list[i].quality)
                    {
                        // `j` is better than `i`
                        list.swap_remove(i);
                        continue 'iloop;
                    }
                }

                // `i` is not that bad
                i += 1;
            }
        }
    }
}
