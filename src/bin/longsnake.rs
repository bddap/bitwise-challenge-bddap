//! A modified version of https://github.com/zesterer/bitwise-examples/blob/main/examples/snake.rs
//! That packs data more densely using non-integer numbers of bits for some elements.

use bitwise_challenge::{Game, Input, Key, Output};
use bitwise_challenge_bddap::cheeky_encoding::{decode, encode};

struct Snake;

const CELLS: u32 = 8;
const CELL: u32 = 32;
const SCORE_H: u32 = 64;
const SCORE_MAX: u8 = 63;
const FIELD_COUNT: usize = 26;

struct Data {
    pos: [u32; 2],
    dir: u32,
    score: u8,
    fruit_pos: [u32; 2],
    tail: [u8; 19],
    is_dead: bool,
}

fn rand(seed: u64) -> u64 {
    let x = (seed.wrapping_mul(182099923) ^ seed).wrapping_add(8301719803) ^ seed;
    x ^ seed ^ x.wrapping_div(21273)
}

impl Data {
    fn to_u64s(&self) -> [u64; FIELD_COUNT] {
        [
            self.pos[0] as u64,
            self.pos[1] as u64,
            self.dir as u64,
            self.score as u64,
            self.fruit_pos[0] as u64,
            self.fruit_pos[1] as u64,
            self.tail[0] as u64,
            self.tail[1] as u64,
            self.tail[2] as u64,
            self.tail[3] as u64,
            self.tail[4] as u64,
            self.tail[5] as u64,
            self.tail[6] as u64,
            self.tail[7] as u64,
            self.tail[8] as u64,
            self.tail[9] as u64,
            self.tail[10] as u64,
            self.tail[11] as u64,
            self.tail[12] as u64,
            self.tail[13] as u64,
            self.tail[14] as u64,
            self.tail[15] as u64,
            self.tail[16] as u64,
            self.tail[17] as u64,
            self.tail[18] as u64,
            self.is_dead as u64,
        ]
    }

    const CARDINALITIES: [u64; FIELD_COUNT] = [
        CELLS as u64,
        CELLS as u64,
        4,
        SCORE_MAX as u64 + 1,
        CELLS as u64,
        CELLS as u64,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        4,
        2,
    ];

    fn from_u64s(data: [u64; FIELD_COUNT]) -> Self {
        Self {
            pos: [data[0] as u32, data[1] as u32],
            dir: data[2] as u32,
            score: data[3] as u8,
            fruit_pos: [data[4] as u32, data[5] as u32],
            tail: [
                data[6] as u8,
                data[7] as u8,
                data[8] as u8,
                data[9] as u8,
                data[10] as u8,
                data[11] as u8,
                data[12] as u8,
                data[13] as u8,
                data[14] as u8,
                data[15] as u8,
                data[16] as u8,
                data[17] as u8,
                data[18] as u8,
                data[19] as u8,
                data[20] as u8,
                data[21] as u8,
                data[22] as u8,
                data[23] as u8,
                data[24] as u8,
            ],
            is_dead: data[25] == 1,
        }
    }
}

fn make_state(data: Data) -> u64 {
    encode(&data.to_u64s(), &Data::CARDINALITIES)
}

fn from_state(state: u64) -> Data {
    Data::from_u64s(decode(state, &Data::CARDINALITIES))
}

impl Game for Snake {
    const NAME: &'static str = "Snake";
    const WIDTH: usize = (CELLS * CELL) as usize;
    const HEIGHT: usize = (CELLS * CELL + SCORE_H) as usize;

    fn init() -> u64 {
        make_state(Data {
            pos: [4, 4],
            dir: 0,
            score: 0,
            fruit_pos: [5, 3],
            tail: [0; 19],
            is_dead: false,
        })
    }

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
        let mut data = from_state(prev);

        fn move_dir(mut pos: [u32; 2], dir: u32) -> [u32; 2] {
            match dir {
                0 => pos[0] = (pos[0] + 1) % CELLS,
                1 => pos[1] = (pos[1] + CELLS - 1) % CELLS,
                2 => pos[0] = (pos[0] + CELLS - 1) % CELLS,
                3 => pos[1] = (pos[1] + 1) % CELLS,
                _ => unreachable!(),
            }
            pos
        }

        if input.tick() % 15 == 0 && !data.is_dead {
            data.pos = move_dir(data.pos, data.dir);

            if data.pos == data.fruit_pos {
                let x = rand(input.tick());
                let y = rand(x);
                data.fruit_pos = [x as u32 % CELLS, y as u32 % CELLS];
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

        if !data.is_dead {
            // Draw snake
            let mut segment = data.pos;
            for i in 0..data.score + 1 {
                if i > 0 && segment == data.pos {
                    data.is_dead = true;
                    data.score = 0;
                }

                output.rect(
                    (segment[0] * CELL) as i32,
                    (segment[1] * CELL + SCORE_H) as i32,
                    CELL,
                    CELL,
                    [0, i * 10, 255 - i * 10],
                );
                if let Some(dir) = data.tail.get(i as usize) {
                    segment = move_dir(segment, *dir as u32);
                } else {
                    break;
                }
            }
        }

        // Draw fruit
        output.rect(
            (data.fruit_pos[0] * CELL) as i32,
            (data.fruit_pos[1] * CELL + SCORE_H) as i32,
            CELL,
            CELL,
            [0, 255, 0],
        );

        // Draw score
        if data.is_dead {
            output.rect(
                0,
                0,
                CELLS * CELL,
                SCORE_H,
                [0, 0, if input.tick() % 16 < 8 { 255 } else { 0 }],
            );
            data.score += 1;
            if data.score == SCORE_MAX {
                return Self::init();
            }
        } else {
            output.rect(0, 0, CELLS * CELL, SCORE_H, [100, 100, 100]);
            output.rect(0, 0, data.score as u32 * 5, SCORE_H, [0, 255, 0]);
        }

        make_state(data)
    }
}

fn main() {
    Snake::run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_cardinality() {
        let max = Data::CARDINALITIES.map(|cardinality| cardinality - 1);
        assert_eq!(Data::from_u64s(max).to_u64s(), max);
        let encoded = encode(&max, &Data::CARDINALITIES);
        let decoded = decode(encoded, &Data::CARDINALITIES);
        assert_eq!(decoded, max);
    }

    #[test]
    fn wasted_data() {
        let wasted_space = 2 * 2 * 2 * 2 * 2;
        let product = Data::CARDINALITIES
            .iter()
            .copied()
            .map(Into::<u128>::into)
            .product::<u128>();
        assert_eq!(product * wasted_space, u64::MAX as u128 + 1);
    }
}
