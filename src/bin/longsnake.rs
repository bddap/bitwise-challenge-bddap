//! A modified version of https://github.com/zesterer/bitwise-examples/blob/main/examples/snake.rs
//! That packs data more densely using non-integer numbers of bits for some elements.

use std::{
    iter::once,
    ops::{Add, Neg},
};

use bitwise_challenge::{Game, Input, Key, Output};
use bitwise_challenge_bddap::cheeky_encoding::{decode, encode};

struct Snake;

const CELLS: u32 = 8;
const CELL: u32 = 32;
const SCORE_H: u32 = 64;
const SCORE_MAX: u8 = 19;
const FIELD_COUNT: usize = 26;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Turn(i8);

impl Turn {
    const LEFT: Self = Turn(-1);
    const STRAIGHT: Self = Turn(0);
    const RIGHT: Self = Turn(1);
}

#[derive(PartialEq, Copy, Clone, Debug)]
struct Direction(i8);

impl Direction {
    const EAST: Self = Direction(0);
    const NORTH: Self = Direction(1);
    const WEST: Self = Direction(2);
    const SOUTH: Self = Direction(3);

    fn front(self) -> [i32; 2] {
        match self.0 {
            0 => [1, 0],  // Right
            1 => [0, -1], // Up
            2 => [-1, 0], // Left
            3 => [0, 1],  // Down
            _ => unreachable!(),
        }
    }

    fn relative(self, other: Self) -> Option<Turn> {
        match (self.0 - other.0 + 4) % 4 {
            0 => Some(Turn::STRAIGHT),
            1 => Some(Turn::RIGHT),
            2 => None, // Opposite
            3 => Some(Turn::LEFT),
            _ => unreachable!(),
        }
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Direction((self.0 + 2) % 4)
    }
}

impl Add<Turn> for Direction {
    type Output = Self;

    fn add(self, turn: Turn) -> Self::Output {
        Self((self.0 + turn.0 + 4) % 4)
    }
}

struct Data {
    pos: [u32; 2],
    dir: Direction,
    score: u8,
    fruit_pos: [u32; 2],
    tail: [Turn; SCORE_MAX as usize],
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
            self.dir.0 as u64,
            self.score as u64,
            self.fruit_pos[0] as u64,
            self.fruit_pos[1] as u64,
            (self.tail[0].0 + 1) as u64,
            (self.tail[1].0 + 1) as u64,
            (self.tail[2].0 + 1) as u64,
            (self.tail[3].0 + 1) as u64,
            (self.tail[4].0 + 1) as u64,
            (self.tail[5].0 + 1) as u64,
            (self.tail[6].0 + 1) as u64,
            (self.tail[7].0 + 1) as u64,
            (self.tail[8].0 + 1) as u64,
            (self.tail[9].0 + 1) as u64,
            (self.tail[10].0 + 1) as u64,
            (self.tail[11].0 + 1) as u64,
            (self.tail[12].0 + 1) as u64,
            (self.tail[13].0 + 1) as u64,
            (self.tail[14].0 + 1) as u64,
            (self.tail[15].0 + 1) as u64,
            (self.tail[16].0 + 1) as u64,
            (self.tail[17].0 + 1) as u64,
            (self.tail[18].0 + 1) as u64,
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
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        3,
        2,
    ];

    fn from_u64s(data: [u64; FIELD_COUNT]) -> Self {
        Self {
            pos: [data[0] as u32, data[1] as u32],
            dir: Direction(data[2] as i8),
            score: data[3] as u8,
            fruit_pos: [data[4] as u32, data[5] as u32],
            tail: [
                Turn(data[6] as i8 - 1),
                Turn(data[7] as i8 - 1),
                Turn(data[8] as i8 - 1),
                Turn(data[9] as i8 - 1),
                Turn(data[10] as i8 - 1),
                Turn(data[11] as i8 - 1),
                Turn(data[12] as i8 - 1),
                Turn(data[13] as i8 - 1),
                Turn(data[14] as i8 - 1),
                Turn(data[15] as i8 - 1),
                Turn(data[16] as i8 - 1),
                Turn(data[17] as i8 - 1),
                Turn(data[18] as i8 - 1),
                Turn(data[19] as i8 - 1),
                Turn(data[20] as i8 - 1),
                Turn(data[21] as i8 - 1),
                Turn(data[22] as i8 - 1),
                Turn(data[23] as i8 - 1),
                Turn(data[24] as i8 - 1),
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

fn move_dir(pos: [u32; 2], dir: Direction) -> [u32; 2] {
    const CE: i32 = CELLS as i32;
    let [xd, yd] = dir.front();
    let [x, y] = pos.map(|c| c as i32);
    [x + xd, y + yd].map(|c| ((c + CE) % CE) as u32)
}

fn rasterize_snek(
    head: [u32; 2],
    facing: Direction,
    tail: &[Turn],
) -> impl Iterator<Item = [u32; 2]> {
    let mut pos = head;
    let mut dir = -facing;
    once(pos).chain(tail.iter().map(move |turn| {
        dir = dir + *turn;
        pos = move_dir(pos, dir);
        pos
    }))
}

impl Game for Snake {
    const NAME: &'static str = "Snake";
    const WIDTH: usize = (CELLS * CELL) as usize;
    const HEIGHT: usize = (CELLS * CELL + SCORE_H) as usize;

    fn init() -> u64 {
        make_state(Data {
            pos: [4, 4],
            dir: Direction(0),
            score: 0,
            fruit_pos: [5, 3],
            tail: [Turn(0); 19],
            is_dead: false,
        })
    }

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
        let mut data = from_state(prev);

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
            } else {
                return make_state(data);
            }
        }

        if input.tick() % 15 == 0 {
            data.pos = move_dir(data.pos, data.dir);

            if data.pos == data.fruit_pos {
                let x = rand(input.tick());
                let y = rand(x);
                data.fruit_pos = [x as u32 % CELLS, y as u32 % CELLS];
                data.score += 1;
            }

            for i in (0..data.tail.len() - 1).rev() {
                data.tail[i + 1] = data.tail[i];
            }
            data.tail[0] = Turn::STRAIGHT;
        }

        let new_dir = if input.is_key_down(Key::Right) {
            Direction::EAST
        } else if input.is_key_down(Key::Left) {
            Direction::WEST
        } else if input.is_key_down(Key::Up) {
            Direction::NORTH
        } else if input.is_key_down(Key::Down) {
            Direction::SOUTH
        } else {
            data.dir
        };

        match data.dir.relative(new_dir) {
            Some(Turn::STRAIGHT) | None => {}
            Some(turn) => {
                data.tail[0] = turn;
                data.dir = new_dir;
            }
        }

        for pos in rasterize_snek(data.pos, data.dir, &data.tail[..data.score as usize]).skip(1) {
            if pos == data.pos {
                data.is_dead = true;
                data.score = 0;
            }
        }

        for (i, pos) in
            rasterize_snek(data.pos, data.dir, &data.tail[..data.score as usize]).enumerate()
        {
            let i = i as u8;
            output.rect(
                (pos[0] * CELL) as i32,
                (pos[1] * CELL + SCORE_H) as i32,
                CELL,
                CELL,
                [0, i * 10, 255 - i * 10],
            );
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
        output.rect(0, 0, CELLS * CELL, SCORE_H, [100, 100, 100]);
        output.rect(0, 0, data.score as u32 * 5, SCORE_H, [0, 255, 0]);

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

    #[test]
    fn parameterized_turns() {
        let test_cases = [
            (Direction::EAST, Direction::NORTH, Some(Turn::LEFT)),
            (Direction::NORTH, Direction::WEST, Some(Turn::LEFT)),
            (Direction::WEST, Direction::SOUTH, Some(Turn::LEFT)),
            (Direction::SOUTH, Direction::EAST, Some(Turn::LEFT)),
            (Direction::EAST, Direction::SOUTH, Some(Turn::RIGHT)),
            (Direction::NORTH, Direction::EAST, Some(Turn::RIGHT)),
            (Direction::WEST, Direction::NORTH, Some(Turn::RIGHT)),
            (Direction::SOUTH, Direction::WEST, Some(Turn::RIGHT)),
            (Direction::EAST, Direction::EAST, Some(Turn::STRAIGHT)),
            (Direction::NORTH, Direction::NORTH, Some(Turn::STRAIGHT)),
            (Direction::WEST, Direction::WEST, Some(Turn::STRAIGHT)),
            (Direction::SOUTH, Direction::SOUTH, Some(Turn::STRAIGHT)),
            (Direction::EAST, Direction::WEST, None),
            (Direction::NORTH, Direction::SOUTH, None),
            (Direction::WEST, Direction::EAST, None),
            (Direction::SOUTH, Direction::NORTH, None),
        ];

        for (from, to, expected) in test_cases {
            assert_eq!(
                from.relative(to),
                expected,
                "Testing from {:?} to {:?}",
                from,
                to
            );
        }
    }
}
