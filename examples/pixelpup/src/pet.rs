//! 小狗的规则：饿、开心、年龄、夜里睡觉。都是纯函数，时间从参数进来（Unix 秒），不在这里读钟。
//! 单元测试就能给任何时间：「过了 8 小时」「走了三天」。

/// 饱和开心都用千分之一点存：满是 100 点，也就是 100_000。
pub const MAX: i64 = 100_000;
/// 每分钟饿多少（千分之一点）：一小时 7.5 点，满的狗 13 个多小时饿空。
pub const FOOD_PER_MIN: i64 = 125;
/// 每分钟不开心多少：一小时 4.8 点。
pub const FUN_PER_MIN: i64 = 80;
/// 离开再久，最多算 12 小时：出门几天回来，狗饿了，但不会一直空着；钟被调乱了也不出怪事。
pub const CAP: i64 = 12 * 3600;
/// 喂一次加多少饱、加多少开心。
pub const FEED_GAIN: (i64, i64) = (35_000, 3_000);
/// 遛一次加多少开心、饿多少。
pub const WALK_GAIN: (i64, i64) = (30_000, 5_000);
/// 低于这个就算饿了、闷了。
pub const LOW: i64 = 25_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pet {
    /// 哪一刻领养的（Unix 秒）：算几天大。
    pub born: i64,
    /// 规则算到了哪一刻（Unix 秒）。只按整分钟往前走，零头留到下次，所以每秒调和隔很久调一次算出来一样。
    pub last: i64,
    pub food: i64,
    pub fun: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Hungry,
    Bored,
    Happy,
}

impl Pet {
    pub fn new(now: i64) -> Pet {
        Pet { born: now, last: now, food: 80_000, fun: 80_000 }
    }

    /// 把 `last` 到 `now` 这段时间补上。回离开了多少秒（没补上限前的真时间）。
    /// 钟往回拨了（`now` 比 `last` 早）：什么都不扣，从现在重新算。
    pub fn catch_up(&mut self, now: i64) -> i64 {
        let gap = now - self.last;
        if gap < 0 {
            self.last = now;
            return 0;
        }
        let mins = gap.min(CAP) / 60;
        self.food = (self.food - mins * FOOD_PER_MIN).max(0);
        self.fun = (self.fun - mins * FUN_PER_MIN).max(0);
        // 超过上限的那段不算了，直接跳到现在。
        self.last = if gap > CAP { now } else { self.last + mins * 60 };
        gap
    }

    pub fn feed(&mut self, now: i64) {
        self.catch_up(now);
        self.food = (self.food + FEED_GAIN.0).min(MAX);
        self.fun = (self.fun + FEED_GAIN.1).min(MAX);
    }

    pub fn walk(&mut self, now: i64) {
        self.catch_up(now);
        self.fun = (self.fun + WALK_GAIN.0).min(MAX);
        self.food = (self.food - WALK_GAIN.1).max(0);
    }

    /// 几天大：领养那天是 0。
    pub fn age_days(&self, now: i64) -> i64 {
        ((now - self.born) / 86_400).max(0)
    }

    pub fn mood(&self) -> Mood {
        if self.food < LOW {
            Mood::Hungry
        } else if self.fun < LOW {
            Mood::Bored
        } else {
            Mood::Happy
        }
    }

    /// 存档：「1,领养,算到,饱,开心」。开头的 1 是格式的版本。
    pub fn encode(&self) -> String {
        format!("1,{},{},{},{}", self.born, self.last, self.food, self.fun)
    }

    pub fn decode(s: &str) -> Option<Pet> {
        let v: Vec<i64> = s.trim().split(',').map(|x| x.parse().ok()).collect::<Option<_>>()?;
        match v[..] {
            [1, born, last, food, fun] => Some(Pet { born, last, food: food.clamp(0, MAX), fun: fun.clamp(0, MAX) }),
            _ => None,
        }
    }
}

/// 夜里（本地 22:00 到 7:00）小狗睡觉。`hour` 是本地的钟点。
pub fn is_night(hour: u32) -> bool {
    !(7..22).contains(&hour)
}
