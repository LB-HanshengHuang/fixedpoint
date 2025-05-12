pub struct Reader {
    s: String,
    i: usize,
}

impl Reader {

    pub fn new(s: &str, i: usize) -> Reader {
        Reader { s: s.into(), i }
    }

    pub fn cur(&self) -> char {
        if self.i >= self.s.len() {
            return 0 as char;
        }
        self.s.as_bytes()[self.i] as char
    }

    pub fn prev(&self) -> char {
        if self.i == 0 {
            return 0 as char;
        }
        self.s.as_bytes()[self.i - 1] as char
    }

    pub fn len(&self) -> usize {
        self.s.len() - self.i
    }

    pub fn match_c(&mut self, c: char) -> bool {
        if self.cur() == c {
            self.i += 1;
            return true;
        }
        false
    }

    pub fn match_digit(&mut self) -> bool {
        let c = self.cur() as char;
        if '0' <= c && c <= '9' {
            self.i += 1;
            return true;
        }
        false
    }

    pub fn match_str_ignore_case(&mut self, pre: &str) -> bool {
        let boundary = self.i + pre.len();
        if boundary > self.s.len() {
            return false;
        }
        let data = self.s[self.i..boundary].to_lowercase();
        let pre = pre.to_lowercase();
        if data == pre {
            self.i = boundary;
            return true;
        }
        false
    }

    pub fn get_sign(&mut self) -> i8 {
        if self.match_c('-') {
            return super::SIGN_NEG
        }
        self.match_c('+');
        super::SIGN_POS
    }

    pub fn get_coef(&mut self) -> (u64, i32) {
        let mut digits = false;
        let mut before_decimal = false;
        while self.match_c('0') {
            digits = true;
        }
        if self.cur() == '.' && self.len() > 1 {
            digits = false;
        }
        let mut n = 0_u64;
        let mut exp = 0_i32;
        let mut p = super::SHIFTMAX;
        loop {
            let c = self.cur();
            if self.match_digit() {
                if c != '0' {
                    n += (c as u8 - b'0') as u64 * super::POW10[p as usize];
                }
                p -= 1;
            } else if before_decimal {
                // decimal point or end
                exp = (super::SHIFTMAX - p) as i32;
                if self.match_c('.') {
                    break
                }
                before_decimal = false;
                if !digits {
                    while self.match_c('0') {
                        digits = true;
                        exp -= 1;
                    }
                }
            } else {
                break;
            }
        }
        if !digits {
            panic!("numbers require at least one digit");
        }
        (n, exp)
    }

    pub fn get_exp(&mut self) -> i32 {
        let mut e = 0;
        let c = self.cur();
        if c == 'e' || c == 'E' {
            self.i += 1;
            let esign = self.get_sign();
            while self.match_digit() {
                e = e*10 + self.prev() as i32 - '0' as i32;
            }
            e *= esign as i32;
        }
        e
    }
}
