#![feature(dec2flt)]

use std::str::FromStr;
use std::num;
use std::cmp;

mod reader;
use reader::Reader;


#[derive(Clone, Debug)]
pub struct Value {
    coef: u64,
    sign: i8,
    exp: i32,
}

#[derive(PartialEq, Debug)]
pub enum RoundingMode {
    Up,
    Down,
    HalfUp,
}

pub const SIGN_POS_INF: i8 = 2;
pub const SIGN_POS: i8 = 1;
pub const SIGN_ZERO: i8 = 0;
pub const SIGN_NEG: i8 = -1;
pub const SIGN_NEG_INF: i8 = -2;
pub const COEF_MIN: u64 = 1000_0000_0000_0000;
pub const COEF_MAX: u64 = 9999_9999_9999_9999;
pub const DIGITSMAX: u32 = 16;
pub const SHIFTMAX: u32 = DIGITSMAX - 1;
pub const MAX_LEADING_ZEROS: u32 = 19;

// Common values
pub const ZERO: Value = Value{coef: 0, sign: SIGN_ZERO, exp: 0};
pub const ONE: Value = Value{coef: COEF_MIN, sign: SIGN_POS, exp: 0};
pub const NEG_ONE:Value = Value{coef: COEF_MIN, sign: SIGN_NEG, exp: 0};
pub const POS_INF: Value = Value{coef: 1, sign: SIGN_POS_INF, exp: 0};
pub const NEG_INF: Value = Value{coef: 1, sign: SIGN_NEG_INF, exp: 0};

impl Default for Value {
    fn default() -> Self {
       ZERO 
    }
}

fn get_digits(mut coef: u64) -> Vec<u8> {
    let mut digits: Vec<u8> = vec![0; DIGITSMAX as usize];
    let mut i = SHIFTMAX;
    let mut nd = 0;
    while coef != 0 {
        digits[nd] = b'0' + (coef / POW10[i as usize]) as u8;
        coef %= POW10[i as usize];
        nd += 1;
        i -= 1;
    }
    return digits[0..nd].to_vec();
}

fn ilog10(x: u64) -> u32 {
    if x == 0 {
        return 0
    }
    let y = ((19 * (63 - x.leading_zeros())) >> 6) as usize;
    if y < 18 && x >= POW10[y + 1] {
        return (y + 1) as u32
    }
    y as u32
}

fn max_shift(x: u64) -> u32 {
   let i = ilog10(x);
   if i > SHIFTMAX {
       return 0
   }
   return SHIFTMAX - i
}

pub fn inf(sign: i8) -> Value {
    if sign == SIGN_POS_INF {
        return POS_INF
    } else if sign == SIGN_NEG_INF {
        return NEG_INF
    }
    ZERO
}

const POW10F: [f64; 17] = [
    1.0,
    10.0,
    100.0,
    1000.0,
    10000.0,
    100000.0,
    1000000.0,
    10000000.0,
    100000000.0,
    1000000000.0,
    10000000000.0,
    100000000000.0,
    1000000000000.0,
    10000000000000.0,
    100000000000000.0,
    1000000000000000.0,
    10000000000000000.0,
];

const POW10: [u64; 19] = [
    1u64,
    10u64,
    100u64,
    1000u64,
    10000u64,
    100000u64,
    1000000u64,
    10000000u64,
    100000000u64,
    1000000000u64,
    10000000000u64,
    100000000000u64,
    1000000000000u64,
    10000000000000u64,
    100000000000000u64,
    1000000000000000u64,
    10000000000000000u64,
    100000000000000000u64,
    1000000000000000000u64,
];

const HALF_POW10: [u64; 16] = [
    0,
    5,
    50,
    500,
    5000,
    50000,
    500000,
    5000000,
    50000000,
    500000000,
    5000000000,
    50000000000,
    500000000000,
    5000000000000,
    50000000000000,
    500000000000000
];

fn new_no_sign_check(sign: i8, mut coef: u64, mut exp: i32) -> Value {
    let mut atmax = false;
    while coef > COEF_MAX {
        coef = (coef + 5) / 10;
        exp += 1;
        atmax = true;
    }

    if !atmax {
        let p = max_shift(coef);
        println!("exp {} p: {}", exp, p);
        coef *= POW10[p as usize];
        exp -= p as i32;
    }
    Value{coef, sign, exp}
}

const LOG2OF10: f64 = 3.32192809488736234;

impl From<f64> for Value {
    fn from(mut value: f64) -> Self {
        if value.is_infinite() {
            return if value.is_sign_positive() {
                POS_INF
            } else {
                NEG_INF
            };
        }
        if value.is_nan() {
            panic!("cannot convert from NaN");
        }
        if value == 0.0 {
            return ZERO;
        }
        let mut sign = SIGN_POS;
        if value < 0.0 {
            value = -value;
            sign = SIGN_NEG;
        }
        let trunc = value.trunc();
        if trunc == value {
            return new_no_sign_check(sign, value as u64, 0);
        }
        let e = ((value - trunc) / LOG2OF10) as usize;
        let c = if e >= 16 {
            value / POW10F[e-16] + 0.5
        } else {
            println!("e: {}", e);
            value * POW10F[16-e] + 0.5
        };
        new_no_sign_check(sign, c as u64, e as i32)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        if value == 0 {
            return ZERO;
        }
        new_no_sign_check(SIGN_POS, value, DIGITSMAX as i32)
    }
}

impl From<i32> for Value {
    fn from(mut value: i32) -> Self {
        if value == 0 {
            return ZERO;
        }
        let mut sign = SIGN_POS;
        if value < 0 {
            value = -value;
            sign = SIGN_NEG;
        }
        new_no_sign_check(sign, value as u64, DIGITSMAX as i32)
    }
}

impl From<i64> for Value {
    fn from(mut value: i64) -> Self {
        if value == 0 {
            return ZERO;
        }
        let mut sign = SIGN_POS;
        if value < 0 {
            value = -value;
            sign = SIGN_NEG;
        }
        new_no_sign_check(sign, value as u64, DIGITSMAX as i32)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.sign == other.sign && self.exp == other.exp && self.coef == other.coef
    }
}
impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if self.sign < other.sign {
            return Some(cmp::Ordering::Less);
        }
        if self.sign > other.sign {
            return Some(cmp::Ordering::Greater);
        }
        if self == other {
            return Some(cmp::Ordering::Equal);
        }
        let sign = self.sign;
        if sign == SIGN_POS_INF || sign == SIGN_NEG_INF || sign == SIGN_ZERO {
            return Some(cmp::Ordering::Equal);
        }
        if self.exp < other.exp {
            return Some(if -sign > 0 {
                cmp::Ordering::Greater
            } else {
                cmp::Ordering::Less
            });
        }
        if self.exp > other.exp {
            return Some(if sign > 0 {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            });
        }
        if self.coef < other.coef {
            return Some(if -sign > 0 {
                cmp::Ordering::Greater
            } else {
                cmp::Ordering::Less
            });
        }
        if self.coef > other.coef {
            return Some(if sign > 0 {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            });
        }
        Some(cmp::Ordering::Equal)
    }
}

impl Value {

    pub fn trunc(&self) -> Self {
        self.integer(RoundingMode::Down)
    }

    fn integer(&self, mode: RoundingMode) -> Value {
        if self.sign == SIGN_ZERO || self.sign == SIGN_POS_INF || self.sign == SIGN_NEG_INF || self.exp >= DIGITSMAX as i32 {
            return self.clone();
        }
        if self.exp <= 0 {
            if mode == RoundingMode::Up || (mode == RoundingMode::HalfUp && self.exp == 0 && self.coef >= ONE.coef * 5) {
                return Value::raw(self.sign, ONE.coef, self.exp+1);
            }
            return ZERO
        }
        let e = DIGITSMAX - self.exp as u32;
        let frac = self.coef % POW10[e as usize];
        if frac == 0 {
            return self.clone();
        }
        let i = self.coef - frac;
        if (mode == RoundingMode::Up && frac > 0) || (mode == RoundingMode::HalfUp && frac >= HALF_POW10[e as usize]) {
            return Value::raw(self.sign, i + POW10[e as usize], self.exp);
        }
        Value::raw(self.sign, i, self.exp)
    }

    // alias of exp
    pub fn num_int_digits(&self) -> i32 {
        self.exp
    }

    pub fn num_digits(&self) -> i32 {
        let mut nd = 0;
        let mut coef = self.coef;
        let mut i = SHIFTMAX;
        while coef != 0 && coef < POW10[i as usize] {
            i -= 1;
        }
        while coef != 0 {
            coef %= POW10[i as usize];
            i -= 1;
            nd += 1;
        }
        nd
    }

    pub fn num_fractional_digits(&self) -> i32 {
        self.num_digits() - self.exp
    }


    pub fn new(mut coef: u64, sign: i8, mut exp: i32) -> Self {
        if sign != SIGN_POS && sign != SIGN_NEG || coef == 0 {
            Self::inf(sign)
        } else {
            let atmax = false;
            while coef > COEF_MAX {
                coef = (coef + 5) / 10;
                exp += 1;
            }
            if !atmax {
                let p = max_shift(coef);
                coef *= POW10[p as usize];
                exp -= p as i32;
            }
            Self::raw(sign, coef, exp)
        }
    } 

    pub fn raw(sign: i8, coef: u64, exp: i32) -> Self {
        Value { sign, coef, exp }
    }

    pub fn inf(sign: i8) -> Self {
        if sign == SIGN_POS_INF {
            POS_INF
        } else if sign == SIGN_NEG_INF {
            NEG_INF
        } else {
            ZERO
        }
    }
}

#[derive(Debug)]
pub struct ParseFloatError {
    kind: num::IntErrorKind
}

impl std::fmt::Display for ParseFloatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseFloatError: {:?}", self.kind)
    }
}

impl std::error::Error for ParseFloatError {}

impl FromStr for Value {
    type Err = ParseFloatError;
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let length = s.len();
        if length == 0 {
            return Err(ParseFloatError{
                kind: num::IntErrorKind::Empty
            });
        }
        let is_percentage = s.ends_with('%');
        if is_percentage {
            s = &s[..length - 1];
        }

        let r = &mut Reader::new(s, 0);
        let sign = r.get_sign();
        if r.match_str_ignore_case("inf") {
            return Ok(Value::inf(sign));
        }
        let (mut coef, mut exp) = r.get_coef();
        exp += r.get_exp();
        if r.len() != 0 { // didin't consume entire string
            return Err(ParseFloatError{
                kind: num::IntErrorKind::InvalidDigit
            });
        } else if coef == 0 || exp < i8::MIN as i32 {
            return Ok(ZERO);
        } else if exp > i8::MAX as i32 {
            return Ok(Value::inf(sign));
        }
        if is_percentage {
            exp -= 2;
        }
        let mut atmax = false;
        while coef > COEF_MAX {
            coef = (coef + 5) / 10;
            exp += 1;
            atmax = true;
        }
        if !atmax {
            let p = max_shift(coef);
            coef *= POW10[p as usize];
            exp -= p as i32;
        }
        Ok(Value::raw(sign, coef, exp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        let v = Value::from(123456789);
        assert_eq!(v.sign, SIGN_POS);
        let digits = get_digits(v.coef);
        assert_eq!(digits, "123456789".as_bytes());
        assert_eq!(v.exp, 9);

        let v1 = Value::from(-123456789);
        assert_eq!(v1.sign, SIGN_NEG);
        assert_eq!(v1.exp, 9);
        assert!(v1 < v);

        let v = Value::from(0);
        assert_eq!(v.coef, 0);
        assert_eq!(v.sign, SIGN_ZERO);
        assert_eq!(v.exp, 0);
        assert_eq!(v, ZERO);

        let v1 = Value::from(0.1);
        assert!(v1 > ZERO);
    }
}
