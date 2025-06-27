//! A modified version of https://github.com/zesterer/bitwise-examples/blob/main/examples/snake.rs
//! That packs data more densely to achieve a longer snake.

use bitwise_challenge::{Game, Input, Key, Output};
use bitwise_challenge_bddap::cheeky_encoding::{decode, encode};

pub struct Snake;

const CELLS: u8 = 8;
const CELL: u8 = 32;
const SCORE_H: u8 = 64;

#[derive(Clone, Default)]
struct Data {
    pos: [u8; 2],
    dir: u8,
    score: u8,
    fruit_pos: [u8; 2],
    tail: [u8; 19],
    is_dead: u8,
    padding: u64,
}

impl Data {
    fn dead(&self) -> bool {
        self.is_dead != 0
    }

    fn set_dead(&mut self, dead: bool) {
        self.is_dead = if dead { 1 } else { 0 };
    }

    fn to_u64s_and_cardinalities(&self) -> [[u64; 27]; 2] {
        let zipped = [
            (self.pos[0] as u64, CELLS as u64),
            (self.pos[1] as u64, CELLS as u64),
            (self.dir as u64, 4),
            (self.score as u64, SCORE_H as u64),
            (self.fruit_pos[0] as u64, CELLS as u64),
            (self.fruit_pos[1] as u64, CELLS as u64),
            (self.tail[0] as u64, 4),
            (self.tail[1] as u64, 4),
            (self.tail[2] as u64, 4),
            (self.tail[3] as u64, 4),
            (self.tail[4] as u64, 4),
            (self.tail[5] as u64, 4),
            (self.tail[6] as u64, 4),
            (self.tail[7] as u64, 4),
            (self.tail[8] as u64, 4),
            (self.tail[9] as u64, 4),
            (self.tail[10] as u64, 4),
            (self.tail[11] as u64, 4),
            (self.tail[12] as u64, 4),
            (self.tail[13] as u64, 4),
            (self.tail[14] as u64, 4),
            (self.tail[15] as u64, 4),
            (self.tail[16] as u64, 4),
            (self.tail[17] as u64, 4),
            (self.tail[18] as u64, 4),
            (self.is_dead as u64, 2),
            (self.padding, 2 * 2 * 2 * 2 * 2), // wasting 5 bits!
        ];
        for (value, cardinality) in zipped {
            debug_assert!(value < cardinality);
        }
        [
            zipped.map(|(value, _)| value),
            zipped.map(|(_, cardinality)| cardinality),
        ]
    }

    fn to_u64s(&self) -> [u64; 27] {
        self.to_u64s_and_cardinalities()[0]
    }

    fn cardinalities() -> [u64; 27] {
        Data::default().to_u64s_and_cardinalities()[1]
    }

    fn from_u64s(data: [u64; 27]) -> Self {
        Self {
            pos: [data[0] as u8, data[1] as u8],
            dir: data[2] as u8,
            score: data[3] as u8,
            fruit_pos: [data[4] as u8, data[5] as u8],
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
            is_dead: data[25] as u8,
            padding: data[26],
        }
    }

    fn encode(&self) -> u64 {
        encode(&self.to_u64s(), &Data::cardinalities())
    }

    fn decode(state: u64) -> Data {
        Data::from_u64s(decode(state, &Data::cardinalities()))
    }
}

#[cfg(test)]
#[test]
fn test_max_cardinality() {
    let max = Data::cardinalities().map(|cardinality| cardinality - 1);
    assert_eq!(Data::from_u64s(max).to_u64s(), max);
    let encoded = encode(&max, &Data::cardinalities());
    assert_eq!(encoded, u64::MAX, "use the whole u64!");
    let decoded = decode(encoded, &Data::cardinalities());
    assert_eq!(decoded, max);
}

fn rand(seed: u64) -> u64 {
    let x = (seed.wrapping_mul(182099923) ^ seed).wrapping_add(8301719803) ^ seed;
    x ^ seed ^ x.wrapping_div(21273)
}

impl Game for Snake {
    const NAME: &'static str = "Snake";
    const WIDTH: usize = CELLS as usize * CELL as usize;
    const HEIGHT: usize = CELLS as usize * CELL as usize + SCORE_H as usize;

    fn init() -> u64 {
        Data {
            pos: [4, 4],
            dir: 0,
            score: 0,
            fruit_pos: [5, 3],
            tail: [0; 19],
            is_dead: 0,
            padding: 0,
        }
        .encode()
    }

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
        let mut data = Data::decode(prev);

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

        data.encode()
    }
}

fn main() {
    Snake::run()
}
