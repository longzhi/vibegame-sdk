//! 出题的规则：只算数，不画、不发声。测试直接调这里的函数。

/// 一局几道题。
pub const ROUND: usize = 10;

/// 难度：10 以内或 20 以内的加减法，不出负数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Ten,
    Twenty,
}

impl Level {
    pub const ALL: [Level; 2] = [Level::Ten, Level::Twenty];

    /// 题里和答案最大是几。
    pub fn max(self) -> u32 {
        match self {
            Level::Ten => 10,
            Level::Twenty => 20,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// 一道题：`a + b` 或 `a - b`，三个选项，`right` 是对的那个在第几个（0 左、1 上、2 右）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Question {
    pub a: u32,
    pub b: u32,
    pub plus: bool,
    pub choices: [u32; 3],
    pub right: usize,
}

impl Question {
    pub fn answer(&self) -> u32 {
        if self.plus {
            self.a + self.b
        } else {
            self.a - self.b
        }
    }

    /// 等号左边：「7 + 5 =」。
    pub fn left(&self) -> String {
        format!("{} {} {} =", self.a, if self.plus { '+' } else { '-' }, self.b)
    }
}

/// 自己的随机数（线性同余）。同一个种子每次出一样的题。
pub struct Rng(pub u32);

impl Rng {
    /// 0 到 n-1。
    pub fn below(&mut self, n: u32) -> u32 {
        self.0 = self.0.wrapping_mul(1103515245).wrapping_add(12345);
        (self.0 >> 16) % n.max(1)
    }
}

/// 一局的种子：固定的数，加上这是第几局、哪一档、在标题上等了几帧。
/// 同样的存档和按键每次出一样的题；每开一局题都不一样。
pub fn seed(rounds: u32, level: Level, wait: u32) -> u32 {
    let mut s = 0x5eed_1234u32;
    for v in [rounds, level as u32, wait] {
        s = (s ^ v).wrapping_mul(0x9e37_79b9).rotate_left(13);
    }
    s
}

/// 出一道题。
pub fn question(rng: &mut Rng, level: Level) -> Question {
    let max = level.max();
    // 20 以内那一档，和或者被减数都过 10，和 10 以内的题分得开。
    let low = if level == Level::Twenty { 11 } else { 2 };
    let top = rng.below(max - low + 1) + low;
    let plus = rng.below(2) == 0;
    let (a, b) = if plus {
        let a = rng.below(top - 1) + 1;
        (a, top - a)
    } else {
        (top, rng.below(top) + 1)
    };
    let mut q = Question { a, b, plus, choices: [0; 3], right: 0 };
    let ans = q.answer();
    // 错的选项离对的差 1 到 3，不出 0 到 max 之外的数。
    let mut near: Vec<u32> = (1..=3).flat_map(|d| [ans + d, ans.wrapping_sub(d)]).filter(|&v| v <= max).collect();
    let mut wrong = [0; 2];
    for w in &mut wrong {
        *w = near.remove(rng.below(near.len() as u32) as usize);
    }
    q.right = rng.below(3) as usize;
    let mut it = wrong.into_iter();
    for (i, c) in q.choices.iter_mut().enumerate() {
        *c = if i == q.right { ans } else { it.next().unwrap() };
    }
    q
}

/// 出一局：十道题，同一局里不出一样的题。
pub fn round(rng: &mut Rng, level: Level) -> Vec<Question> {
    let mut qs: Vec<Question> = Vec::with_capacity(ROUND);
    while qs.len() < ROUND {
        let q = question(rng, level);
        if !qs.iter().any(|p| (p.a, p.b, p.plus) == (q.a, q.b, q.plus)) {
            qs.push(q);
        }
    }
    qs
}

/// 这一局得了 `stars` 颗，最好成绩变成多少。
pub fn better(best: Option<u8>, stars: u8) -> Option<u8> {
    Some(best.map_or(stars, |b| b.max(stars)))
}
