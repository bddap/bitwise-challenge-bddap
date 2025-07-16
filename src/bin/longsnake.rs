use std::{
    iter::once,
    ops::{Add, Neg},
};

use bitwise_challenge::{Game, Input, Key, Output};
use bitwise_challenge_bddap::cheeky_encoding::{decode, encode};

const CELLS: u32 = 8;
const CELL: u32 = 32;
const SCORE_H: u32 = 64;
const SCORE_MAX: u8 = 19;
const FIELD_COUNT: usize = 27;

// Easter egg: it is impossible to represent a 180 degree turn using
// this structure so two quick subsequent turns will rotate the entire
// the entire tail. We could gaurd against this, but the maneuver is
// challenging to pull off so let's leave in for fun.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum Turn {
    Left = 0,
    Straight = 1,
    Right = 2,
}

impl From<u8> for Turn {
    fn from(value: u8) -> Self {
        match value {
            0 => Turn::Left,
            1 => Turn::Straight,
            2 => Turn::Right,
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    East = 0,
    North = 1,
    West = 2,
    South = 3,
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Direction::East,
            1 => Direction::North,
            2 => Direction::West,
            3 => Direction::South,
            _ => unreachable!(),
        }
    }
}

impl Direction {
    fn front(self) -> [i32; 2] {
        match self {
            Self::East => [1, 0],
            Self::North => [0, -1],
            Self::West => [-1, 0],
            Self::South => [0, 1],
        }
    }

    fn relative(self, other: Self) -> Option<Turn> {
        let diff = (self as u8 + 4 - other as u8) % 4;
        match diff {
            0 => Some(Turn::Straight),
            1 => Some(Turn::Right),
            2 => None,
            3 => Some(Turn::Left),
            _ => unreachable!(),
        }
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (self as u8 + 2).into()
    }
}

impl Add<Turn> for Direction {
    type Output = Self;

    fn add(self, turn: Turn) -> Self::Output {
        (self as u8 + turn as u8 + 3).into()
    }
}

struct Snake;

struct Data {
    pos: [u32; 2],
    dir: Direction,
    score: u8,
    fruit_pos: [u32; 2],
    tail: [Turn; SCORE_MAX as usize],
    is_dead: bool,
    pad: u64,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            pos: [4, 4],
            dir: Direction::East,
            score: 0,
            fruit_pos: [5, 3],
            tail: [Turn::Straight; 19],
            is_dead: false,
            pad: 0,
        }
    }
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
            self.dir as i8 as u64,
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
            self.pad,
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
        24217,
    ];

    fn from_u64s(data: [u64; FIELD_COUNT]) -> Self {
        Self {
            pos: [data[0] as u32, data[1] as u32],
            dir: (data[2] as u8).into(),
            score: data[3] as u8,
            fruit_pos: [data[4] as u32, data[5] as u32],
            tail: [
                (data[6] as u8).into(),
                (data[7] as u8).into(),
                (data[8] as u8).into(),
                (data[9] as u8).into(),
                (data[10] as u8).into(),
                (data[11] as u8).into(),
                (data[12] as u8).into(),
                (data[13] as u8).into(),
                (data[14] as u8).into(),
                (data[15] as u8).into(),
                (data[16] as u8).into(),
                (data[17] as u8).into(),
                (data[18] as u8).into(),
                (data[19] as u8).into(),
                (data[20] as u8).into(),
                (data[21] as u8).into(),
                (data[22] as u8).into(),
                (data[23] as u8).into(),
                (data[24] as u8).into(),
            ],
            is_dead: data[25] == 1,
            pad: data[26],
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
    let [xd, yd] = dir.front();
    let [x, y] = pos.map(|c| c as i32);
    let ce = CELLS as i32;
    [x + xd, y + yd].map(|c| ((c + ce) % ce) as u32)
}

impl Game for Snake {
    const NAME: &'static str = "Snake";
    const WIDTH: usize = (CELLS * CELL) as usize;
    const HEIGHT: usize = (CELLS * CELL + SCORE_H) as usize;

    fn init() -> u64 {
        make_state(Default::default())
    }

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64 {
        let mut data = from_state(prev);
        data.update(input.tick());
        data.handle_input(input);
        data.render(output, input.tick());
        make_state(data)
    }
}

impl Data {
    fn segment_positions(&self) -> impl Iterator<Item = [u32; 2]> + '_ {
        let head = once(self.pos);

        let mut pos = self.pos;
        let mut dir = -self.dir;
        let tail = self.tail.iter().take(self.score as usize).map(move |turn| {
            dir = dir + *turn;
            pos = move_dir(pos, dir);
            pos
        });

        head.chain(tail)
    }

    fn handle_input(&mut self, input: &Input<'_, Snake>) {
        let new_dir = if input.is_key_down(Key::Right) {
            Direction::East
        } else if input.is_key_down(Key::Left) {
            Direction::West
        } else if input.is_key_down(Key::Up) {
            Direction::North
        } else if input.is_key_down(Key::Down) {
            Direction::South
        } else {
            self.dir
        };

        if let Some(turn) = self.dir.relative(new_dir)
            && turn != Turn::Straight
        {
            self.tail[0] = turn;
            self.dir = new_dir;
        }
    }

    fn update(&mut self, tick: u64) {
        if self.is_dead {
            if tick % 10 != 0 {
                // run the death animation a bit longer
                // to allow the player some agony
                return;
            }
            if self.score == 0 {
                *self = Data::default();
            } else {
                self.score -= 1;
            }
            return;
        }

        if tick % 15 == 0 {
            self.pos = move_dir(self.pos, self.dir);

            if self.pos == self.fruit_pos {
                let x = rand(tick);
                let y = rand(x);
                self.fruit_pos = [x as u32 % CELLS, y as u32 % CELLS];
                self.score += 1;
            }

            for i in (0..self.tail.len() - 1).rev() {
                self.tail[i + 1] = self.tail[i];
            }
            self.tail[0] = Turn::Straight;
        }

        if self.segment_positions().skip(1).any(|pos| pos == self.pos) {
            self.is_dead = true;
        }
    }

    fn render(&self, output: &mut Output<'_, Snake>, tick: u64) {
        // snake
        for (i, pos) in self.segment_positions().enumerate() {
            let blue = {
                let t = tick + i as u64;
                let t = t % 16 * 16;
                t as u8 * self.is_dead as u8
            };
            let i = i as u8;
            output.rect(
                (pos[0] * CELL) as i32,
                (pos[1] * CELL + SCORE_H) as i32,
                CELL,
                CELL,
                [blue, i * 10, 255 - i * 10],
            );
        }

        // fruit
        output.rect(
            (self.fruit_pos[0] * CELL) as i32,
            (self.fruit_pos[1] * CELL + SCORE_H) as i32,
            CELL,
            CELL,
            [0, 255, 0],
        );

        // score
        output.rect(0, 0, CELLS * CELL, SCORE_H, [100, 100, 100]);
        output.rect(0, 0, self.score as u32 * 5, SCORE_H, [0, 255, 0]);
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
        let product = Data::CARDINALITIES
            .into_iter()
            .map(Into::<u128>::into)
            .product::<u128>();
        let available = (u64::MAX as u128 / product) - 1;
        let bits: f64 = (product as f64).log2();
        assert_eq!(
            available, 0,
            "you are wasting {bits} bits, {available} available values"
        );
        // dubious correctness ---^, its at least close correct
    }

    #[test]
    fn parameterized_turns() {
        let test_cases = [
            (Direction::East, Direction::North, Some(Turn::Left)),
            (Direction::North, Direction::West, Some(Turn::Left)),
            (Direction::West, Direction::South, Some(Turn::Left)),
            (Direction::South, Direction::East, Some(Turn::Left)),
            (Direction::East, Direction::South, Some(Turn::Right)),
            (Direction::North, Direction::East, Some(Turn::Right)),
            (Direction::West, Direction::North, Some(Turn::Right)),
            (Direction::South, Direction::West, Some(Turn::Right)),
            (Direction::East, Direction::East, Some(Turn::Straight)),
            (Direction::North, Direction::North, Some(Turn::Straight)),
            (Direction::West, Direction::West, Some(Turn::Straight)),
            (Direction::South, Direction::South, Some(Turn::Straight)),
            (Direction::East, Direction::West, None),
            (Direction::North, Direction::South, None),
            (Direction::West, Direction::East, None),
            (Direction::South, Direction::North, None),
        ];

        for (from, to, expected) in test_cases {
            assert_eq!(
                from.relative(to),
                expected,
                "Testing from {from:?} to {to:?}",
            );
        }
    }
}
