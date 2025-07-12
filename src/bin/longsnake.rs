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
const FIELD_COUNT: usize = 26;

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum Turn {
    Left = -1,
    Straight = 0,
    Right = 1,
}

impl From<i8> for Turn {
    fn from(value: i8) -> Self {
        let value = value.wrapping_add(1).rem_euclid(3) - 1;
        match value {
            -1 => Self::Left,
            0 => Self::Straight,
            1 => Self::Right,
            _ => unreachable!(),
        }
    }
}

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    East = 0,
    North = 1,
    West = 2,
    South = 3,
}

impl Direction {
    fn from_int(value: i8) -> Self {
        match value % 4 {
            0 => Self::East,
            1 => Self::North,
            2 => Self::West,
            3 => Self::South,
            _ => unreachable!(),
        }
    }

    fn front(self) -> [i32; 2] {
        match self {
            Self::East => [1, 0],
            Self::North => [0, -1],
            Self::West => [-1, 0],
            Self::South => [0, 1],
        }
    }

    fn relative(self, other: Self) -> Option<Turn> {
        let s = self as i8;
        let o = other as i8;
        match (s - o + 4) % 4 {
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
        Self::from_int(self as i8 + 2)
    }
}

impl Add<Turn> for Direction {
    type Output = Self;

    fn add(self, turn: Turn) -> Self::Output {
        Self::from_int(self as i8 + turn as i8 + 4)
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
            (self.tail[0] as i8 + 1) as u64,
            (self.tail[1] as i8 + 1) as u64,
            (self.tail[2] as i8 + 1) as u64,
            (self.tail[3] as i8 + 1) as u64,
            (self.tail[4] as i8 + 1) as u64,
            (self.tail[5] as i8 + 1) as u64,
            (self.tail[6] as i8 + 1) as u64,
            (self.tail[7] as i8 + 1) as u64,
            (self.tail[8] as i8 + 1) as u64,
            (self.tail[9] as i8 + 1) as u64,
            (self.tail[10] as i8 + 1) as u64,
            (self.tail[11] as i8 + 1) as u64,
            (self.tail[12] as i8 + 1) as u64,
            (self.tail[13] as i8 + 1) as u64,
            (self.tail[14] as i8 + 1) as u64,
            (self.tail[15] as i8 + 1) as u64,
            (self.tail[16] as i8 + 1) as u64,
            (self.tail[17] as i8 + 1) as u64,
            (self.tail[18] as i8 + 1) as u64,
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
            dir: Direction::from_int(data[2] as i8),
            score: data[3] as u8,
            fruit_pos: [data[4] as u32, data[5] as u32],
            tail: [
                (data[6] as i8 - 1).into(),
                (data[7] as i8 - 1).into(),
                (data[8] as i8 - 1).into(),
                (data[9] as i8 - 1).into(),
                (data[10] as i8 - 1).into(),
                (data[11] as i8 - 1).into(),
                (data[12] as i8 - 1).into(),
                (data[13] as i8 - 1).into(),
                (data[14] as i8 - 1).into(),
                (data[15] as i8 - 1).into(),
                (data[16] as i8 - 1).into(),
                (data[17] as i8 - 1).into(),
                (data[18] as i8 - 1).into(),
                (data[19] as i8 - 1).into(),
                (data[20] as i8 - 1).into(),
                (data[21] as i8 - 1).into(),
                (data[22] as i8 - 1).into(),
                (data[23] as i8 - 1).into(),
                (data[24] as i8 - 1).into(),
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
    let [xd, yd] = dir.front();
    let [x, y] = pos.map(|c| c as i32);
    let ce = CELLS as i32;
    [x + xd, y + yd].map(|c| ((c + ce) % ce) as u32)
}

fn rasterize_snek(
    head: [u32; 2],
    facing: Direction,
    tail: &[Turn],
) -> impl Iterator<Item = [u32; 2]> + Clone {
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
            dir: Direction::East,
            score: 0,
            fruit_pos: [5, 3],
            tail: [Turn::Straight; 19],
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
            }
            return make_state(data);
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
            data.tail[0] = Turn::Straight;
        }

        let new_dir = if input.is_key_down(Key::Right) {
            Direction::East
        } else if input.is_key_down(Key::Left) {
            Direction::West
        } else if input.is_key_down(Key::Up) {
            Direction::North
        } else if input.is_key_down(Key::Down) {
            Direction::South
        } else {
            data.dir
        };

        if let Some(turn) = data.dir.relative(new_dir)
            && turn != Turn::Straight
        {
            data.tail[0] = turn;
            data.dir = new_dir;
        }

        let snek_positions = rasterize_snek(data.pos, data.dir, &data.tail[..data.score as usize]);
        for pos in snek_positions.clone().skip(1) {
            if pos == data.pos {
                data.is_dead = true;
                data.score = 0;
            }
        }

        for (i, pos) in snek_positions.enumerate() {
            let i = i as u8;
            output.rect(
                (pos[0] * CELL) as i32,
                (pos[1] * CELL + SCORE_H) as i32,
                CELL,
                CELL,
                [0, i * 10, 255 - i * 10],
            );
        }

        output.rect(
            (data.fruit_pos[0] * CELL) as i32,
            (data.fruit_pos[1] * CELL + SCORE_H) as i32,
            CELL,
            CELL,
            [0, 255, 0],
        );

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
