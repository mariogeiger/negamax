# negamax

- Cargo.toml :
```
[dependencies]
binary_heap_compare = { git = "https://github.com/mariogeiger/negamax" }
```

- src/main.rs :
```
extern crate negamax;
use negamax::GameState;

struct TicTacToe([i32; 3 * 3]);

impl<'a> negamax::GameState<'a> for TicTacToe {
    type It = Vec<TicTacToe>;

    fn value(&self) -> i32 {
        // ...
    }

    //...
```