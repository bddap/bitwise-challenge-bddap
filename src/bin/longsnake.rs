//! A modified version of https://github.com/zesterer/bitwise-examples/blob/main/examples/snake.rs
//! That packs data more densely to achieve a longer snake.

use bitwise_challenge::{Game, Input, Key, Output};

struct Snake;

const CELLS: u8 = 8;
const CELL: u8 = 32;
const SCORE_H: u8 = 64;

fn push(state: &mut u64, value: u64, cardinality: u64) {
    debug_assert!(value < cardinality);
    *state *= cardinality;
    *state += value;
}

fn pop(state: &mut u64, cardinality: u64) -> u64 {
    let ret = *state % cardinality;
    *state /= cardinality;
    ret
}

#[derive(Clone, Default)]
struct Data {
    pos: [u8; 2],
    dir: u8,
    score: u8,
    fruit_pos: [u8; 2],
    tail: [u8; 19],
    is_dead: u8,
}

impl Data {
    fn dead(&self) -> bool {
        self.is_dead != 0
    }

    fn set_dead(&mut self, dead: bool) {
        self.is_dead = if dead { 1 } else { 0 };
    }

    #[allow(clippy::needless_lifetimes)]
    fn mut_fields_and_cards<'a>(&'a mut self) -> [(&'a mut u8, u8); 26] {
        unsafe {
            // Safety:
            // - these references are disjoint
            // - lifetime is explicit to ensure rustc doesn't elide it into something incorrect
            let ptr = self as *mut Self;
            [
                (&mut (*ptr).pos[0], CELLS + 1),
                (&mut (*ptr).pos[1], CELLS + 1),
                (&mut (*ptr).dir, 5),
                (&mut (*ptr).score, SCORE_H + 1),
                (&mut (*ptr).fruit_pos[0], CELLS + 1),
                (&mut (*ptr).fruit_pos[1], CELLS + 1),
                (&mut (*ptr).tail[0], 5),
                (&mut (*ptr).tail[1], 5),
                (&mut (*ptr).tail[2], 5),
                (&mut (*ptr).tail[3], 5),
                (&mut (*ptr).tail[4], 5),
                (&mut (*ptr).tail[5], 5),
                (&mut (*ptr).tail[6], 5),
                (&mut (*ptr).tail[7], 5),
                (&mut (*ptr).tail[8], 5),
                (&mut (*ptr).tail[9], 5),
                (&mut (*ptr).tail[10], 5),
                (&mut (*ptr).tail[11], 5),
                (&mut (*ptr).tail[12], 5),
                (&mut (*ptr).tail[13], 5),
                (&mut (*ptr).tail[14], 5),
                (&mut (*ptr).tail[15], 5),
                (&mut (*ptr).tail[16], 5),
                (&mut (*ptr).tail[17], 5),
                (&mut (*ptr).tail[18], 5),
                (&mut (*ptr).is_dead, 2),
            ]
        }
    }
}
// Uncomment the following lines to enable tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     // assert the product of all cardinalities is less than 2^64
//     #[test]
//     fn test_cardinalities() {
//         let mut product = 1u64;
//         for (_, cardinality) in Data::default().mut_fields_and_cards() {
//             product = product
//                 .checked_mul(cardinality as u64)
//                 .expect("Cardinalities overflowed");
//         }
//         assert!(product < (1u64 << 64));
//     }
// }

fn rand(seed: u64) -> u64 {
    let x = (seed.wrapping_mul(182099923) ^ seed).wrapping_add(8301719803) ^ seed;
    x ^ seed ^ x.wrapping_div(21273)
}

fn make_state(data: Data) -> u64 {
    let mut state = 0;
    let mut data = data;
    let view = data.mut_fields_and_cards();
    let view_len = view.len();
    for (i, (field, cardinality)) in view.into_iter().enumerate() {
        if i == view_len - 1 {
            state += *field as u64;
        } else {
            push(&mut state, *field as u64, cardinality as u64);
        }
    }
    state
}

fn from_state(state: u64) -> Data {
    let mut data = Data::default();
    let mut state = state;
    for (i, (field, cardinality)) in data.mut_fields_and_cards().into_iter().enumerate().rev() {
        if i == 0 {
            *field = state as u8;
        } else {
            *field = pop(&mut state, cardinality as u64) as u8;
        }
    }
    data
}

impl Game for Snake {
    const NAME: &'static str = "Snake";
    const WIDTH: usize = CELLS as usize * CELL as usize;
    const HEIGHT: usize = CELLS as usize * CELL as usize + SCORE_H as usize;

    fn init() -> u64 {
        make_state(Data {
            pos: [4, 4],
            dir: 0,
            score: 0,
            fruit_pos: [5, 3],
            tail: [0; 19],
            is_dead: 0,
        })
    }

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
        let mut data = from_state(prev);

        fn move_dir(mut pos: [u8; 2], dir: u8) -> [u8; 2] {
            match dir {
                0 => pos[0] = (pos[0] + 1) % CELLS,
                1 => pos[1] = (pos[1] + CELLS - 1) % CELLS,
                2 => pos[0] = (pos[0] + CELLS - 1) % CELLS,
                3 => pos[1] = (pos[1] + 1) % CELLS,
                _ => unreachable!(),
            }
            pos
        }

        if input.tick() % 15 == 0 && !data.dead() {
            data.pos = move_dir(data.pos, data.dir);

            if data.pos == data.fruit_pos {
                let x = rand(input.tick());
                let y = rand(x);
                data.fruit_pos = [x as u8 % CELLS, y as u8 % CELLS];
                data.score += 1;
            }

            for i in (0..18).rev() {
                data.tail[i + 1] = data.tail[i];
            }
            data.tail[0] = (data.dir as u8 + 2) % 4;
        }

        let new_dir = if input.is_key_down(Key::Right) {
            0
        } else if input.is_key_down(Key::Left) {
            2
        } else if input.is_key_down(Key::Up) {
            1
        } else if input.is_key_down(Key::Down) {
            3
        } else {
            data.dir
        };
        if new_dir != (data.dir + 2) % 4 {
            data.dir = new_dir;
        }

        if !data.dead() {
            // Draw snake
            let mut segment = data.pos;
            for i in 0..data.score + 1 {
                if i > 0 && segment == data.pos {
                    data.set_dead(true);
                    data.score = 0;
                }

                output.rect(
                    (segment[0] * CELL) as i32,
                    (segment[1] * CELL + SCORE_H) as i32,
                    CELL.into(),
                    CELL.into(),
                    [0, i * 10, 255 - i * 10],
                );
                if let Some(dir) = data.tail.get(i as usize) {
                    segment = move_dir(segment, *dir);
                } else {
                    break;
                }
            }
        }

        // Draw fruit
        output.rect(
            (data.fruit_pos[0] * CELL) as i32,
            (data.fruit_pos[1] * CELL + SCORE_H) as i32,
            CELL.into(),
            CELL.into(),
            [0, 255, 0],
        );

        // Draw score
        if data.dead() {
            output.rect(
                0,
                0,
                CELLS as u32 * CELL as u32,
                SCORE_H.into(),
                [0, 0, if input.tick() % 16 < 8 { 255 } else { 0 }],
            );
            data.score += 1;
            if data.score == 63 {
                return Self::init();
            }
        } else {
            output.rect(
                0,
                0,
                CELLS as u32 * CELL as u32,
                SCORE_H.into(),
                [100, 100, 100],
            );
            output.rect(0, 0, data.score as u32 * 5, SCORE_H.into(), [0, 255, 0]);
        }

        make_state(data)
    }
}

fn main() {
    Snake::run()
}
