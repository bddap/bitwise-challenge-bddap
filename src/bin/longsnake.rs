//! A modified version of https://github.com/zesterer/bitwise-examples/blob/main/examples/snake.rs
//! That packs data more densely to achieve a longer snake.

use bitwise_challenge::{Game, Input, Key, Output};

struct Snake;

const CELLS: u8 = 8;
const CELL: u8 = 32;
const SCORE_H: u8 = 64;

fn set(state: &mut u64, value: u64) {
    *state += value;
}

fn ascend(state: &mut u64, next_cardinality: u64) {
    *state *= next_cardinality;
}

fn get(state: u64, cardinality: u64) -> u64 {
    state % cardinality
}

fn descend(state: &mut u64, cardinality: u64) {
    *state /= cardinality;
}

fn encode<const N: usize>(data: &[u64; N], cardinalities: &[u64; N]) -> u64 {
    let mut state = 0;
    for (value, cardinality) in data.iter().zip(cardinalities) {
        debug_assert!(*value < *cardinality);
        ascend(&mut state, *cardinality); // first ascent is always a nop because state is 0
        set(&mut state, *value);
    }
    state
}

fn decode<const N: usize>(state: u64, cardinalities: &[u64; N]) -> [u64; N] {
    let mut result = [0; N];
    let mut state = state;

    for (i, cardinality) in cardinalities.iter().enumerate().rev() {
        result[i] = get(state, *cardinality);
        descend(&mut state, *cardinality);
    }
    result
}

// fn push(state: &mut u64, value: u64, cardinality: u64) {
//     debug_assert!(value < cardinality);
//     *state *= cardinality;
//     *state += value;
// }

// fn pop(state: &mut u64, cardinality: u64) -> u64 {
//     let ret = *state % cardinality;
//     *state /= cardinality;
//     ret
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_no_data() {
        let mut state = 0;

        let unit = 0;
        let cardinality = 1; // 1 possible state constitutes no data

        set(&mut state, unit);
        assert_eq!(get(state, cardinality), unit);
    }

    #[test]
    fn store_one_bit() {
        let cardinality = 2;

        for bit in [0, 1] {
            let mut state = 0;
            set(&mut state, bit);
            assert_eq!(get(state, cardinality), bit);
        }
    }

    #[test]
    fn store_more_than_one_but_less_than_two_bits() {
        let cardinality = 3; // {0, 1, 2}

        for value in [0, 1, 2] {
            let mut state = 0;
            set(&mut state, value);
            assert_eq!(get(state, cardinality), value);
        }
    }

    #[test]
    fn store_one_bit_twice() {
        let cardinality = 2;

        for (a, b) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let mut state = 0;
            set(&mut state, a);
            ascend(&mut state, cardinality);
            set(&mut state, b);
            assert_eq!(get(state, cardinality), b);
            descend(&mut state, cardinality);
            assert_eq!(get(state, cardinality), a);
        }
    }

    #[test]
    fn store_list() {
        check_store_list(&[
            (0, 1), // nothing, zero bits of data being stored
            (0, 2), // one bit
            (1, 2),
            (0, 3), // log2(3) ~= 1.585 bits
            (1, 3),
            (2, 3),
            (0, 4), // log2(4) = 2 bits
            (1, 4),
            (2, 4),
            (3, 4),
            (0, 5), // log2(5) ~= 2.321 bits
            (1, 5),
            (2, 5),
            (3, 5),
            (4, 5),
        ]);
        check_store_list(&[(1, 2); 32]);
        check_store_list(&[(2, 3); 40]); // 64 / 1.585 ~= 40.4
        check_store_list(&[(4, 5); 27]); // 64 / 2.321 ~= 27.5
        check_store_list(&[(u64::MAX - 1, u64::MAX)]);
        check_store_list(&[(u8::MAX.into(), Into::<u64>::into(u8::MAX) + 1); 8]);
        check_store_list(&[(u16::MAX.into(), Into::<u64>::into(u16::MAX) + 1); 4]);
        check_store_list(&[(u32::MAX.into(), Into::<u64>::into(u32::MAX) + 1); 2]);
    }

    fn check_store_list<const N: usize>(data_vs_possibilities: &[(u64, u64); N]) {
        let just_data = data_vs_possibilities.map(|(data, _)| data);
        let just_possibilities = data_vs_possibilities.map(|(_, possibilities)| possibilities);

        let state = encode(&just_data, &just_possibilities);
        let decoded = decode(state, &just_possibilities);
        assert_eq!(decoded, just_data);
    }
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
                (&mut (*ptr).pos[0], CELLS),
                (&mut (*ptr).pos[1], CELLS),
                (&mut (*ptr).dir, 4),
                (&mut (*ptr).score, SCORE_H),
                (&mut (*ptr).fruit_pos[0], CELLS),
                (&mut (*ptr).fruit_pos[1], CELLS),
                (&mut (*ptr).tail[0], 4),
                (&mut (*ptr).tail[1], 4),
                (&mut (*ptr).tail[2], 4),
                (&mut (*ptr).tail[3], 4),
                (&mut (*ptr).tail[4], 4),
                (&mut (*ptr).tail[5], 4),
                (&mut (*ptr).tail[6], 4),
                (&mut (*ptr).tail[7], 4),
                (&mut (*ptr).tail[8], 4),
                (&mut (*ptr).tail[9], 4),
                (&mut (*ptr).tail[10], 4),
                (&mut (*ptr).tail[11], 4),
                (&mut (*ptr).tail[12], 4),
                (&mut (*ptr).tail[13], 4),
                (&mut (*ptr).tail[14], 4),
                (&mut (*ptr).tail[15], 4),
                (&mut (*ptr).tail[16], 4),
                (&mut (*ptr).tail[17], 4),
                (&mut (*ptr).tail[18], 4),
                (&mut (*ptr).is_dead, 2),
            ]
        }
    }
}

#[cfg(test)]
#[test]
fn test_max_cardinality() {
    let mut data = Data::default();
    for (field, cardinality) in data.mut_fields_and_cards() {
        *field = cardinality - 1;
    }
    make_state(data);
}

fn rand(seed: u64) -> u64 {
    let x = (seed.wrapping_mul(182099923) ^ seed).wrapping_add(8301719803) ^ seed;
    x ^ seed ^ x.wrapping_div(21273)
}

fn make_state(data: Data) -> u64 {
    let mut data = data;
    let dats = data.mut_fields_and_cards().map(|(field, _)| *field as u64);
    let cardinalities = data
        .mut_fields_and_cards()
        .map(|(_, cardinality)| cardinality as u64);
    encode(&dats, &cardinalities)
}

fn from_state(state: u64) -> Data {
    let mut data = Data::default();
    let cardinalities = data
        .mut_fields_and_cards()
        .map(|(_, cardinality)| cardinality as u64);
    let decoded = decode(state, &cardinalities);
    for ((field, _), dat) in data.mut_fields_and_cards().into_iter().zip(decoded) {
        *field = dat as u8;
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
                    segment[0] as i32 * CELL as i32,
                    segment[1] as i32 * CELL as i32 + SCORE_H as i32,
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
            data.fruit_pos[0] as i32 * CELL as i32,
            data.fruit_pos[1] as i32 * CELL as i32 + SCORE_H as i32,
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
