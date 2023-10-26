pub mod json_lib2 {
    use std::collections::{HashMap, VecDeque, HashSet};

    #[derive(PartialEq, Clone, Debug)]
    pub enum Token {
        OpenCurly,
        CloseCurly,
        OpenSquare,
        CloseSquare,
        String(String),
        Number(f32),
        Boolean(bool),
        Null,
        Comma,
        Colon,
        Quote,
        OtherChar(char)
    }

    enum Datum {
        Str(String),
        Number(f32),
        Boolean(bool),
        Null,
        Object(HashMap::<String, Datum>),
        Array(Vec<Datum>)
    }

    #[derive(Debug, Clone)]
    struct SourceLocation {
        col: usize,
        row: usize
    }

    #[derive(Debug, Clone)]
    struct LexicalToken {
        token: Token,
        from: SourceLocation,
        to: SourceLocation
    }

    #[derive(Debug)]
    pub enum ParseError {
        UnexpectedToken(LexicalToken),
        UnexpectedEof
    }

    enum ParsePhase {
        FindKey,
        FindKeyOrEnd,
        FindColon,
        FindValue,
        FindCommaOrEnd
    }

    pub struct Json {
        pub root: Option<HashMap<String, Datum>>,
        tokens: VecDeque<LexicalToken>,
    }

    impl Json {
        pub fn new() -> Self {
            Json {
                root: None,
                tokens: VecDeque::<LexicalToken>::new()
            }
        }

        pub fn lex(&mut self, src: &str) {
            let tmp = String::from(src);
            for (lnum, line) in tmp.split("\n").enumerate() {
                let mut iter = line.chars().enumerate().peekable();
                while let Some((cnum, ch)) = iter.peek() {
                    if ch.is_whitespace() {
                        iter.next();
                        continue;
                    }
                    let start_loc = SourceLocation {
                        col: *cnum,
                        row: lnum
                    };
                    let mut cnt = *cnum;
                    let tok = match ch {
                        '{' => Token::OpenCurly,
                        '}' => Token::CloseCurly,
                        '[' => Token::OpenSquare,
                        ']' => Token::CloseSquare,
                        ':' => Token::Colon,
                        ',' => Token::Comma,
                        't' => {
                            iter.next();
                            let mut idx = 0;
                            while let Some((_, ch)) = iter.peek() {
                                match idx {
                                    0 => assert_eq!(*ch, 'r'),
                                    1 => assert_eq!(*ch, 'u'),
                                    2 => { assert_eq!(*ch, 'e'); break; }
                                    _ => {}
                                };
                                idx += 1;
                                iter.next();
                            }
                            Token::Boolean(true)
                        },
                        'f' => {
                            iter.next();
                            let mut idx = 0;
                            while let Some((_, ch)) = iter.peek() {
                                match idx {
                                    0 => assert_eq!(*ch, 'a'),
                                    1 => assert_eq!(*ch, 'l'),
                                    2 => assert_eq!(*ch, 's'),
                                    3 => { assert_eq!(*ch, 'e'); break; }
                                    _ => {}
                                };
                                idx += 1;
                                iter.next();
                            }
                            Token::Boolean(false)
                        },
                        'n' => {
                            iter.next();
                            let mut idx = 0;
                            while let Some((_, ch)) = iter.peek() {
                                match idx {
                                    0 => assert_eq!(*ch, 'u'),
                                    1 => assert_eq!(*ch, 'l'),
                                    2 => { assert_eq!(*ch, 'l'); break; }
                                    _ => {}
                                };
                                idx += 1;
                                iter.next();
                            }
                            Token::Null
                        },
                        '"' => {
                            let mut s = String::new();
                            let mut string_closed = false;
                            iter.next();
                            while let Some((_, ch)) = iter.peek() {
                                if *ch == '"' {
                                    string_closed = true;
                                    break;
                                }
                                s.push(*ch);
                                cnt += 1;
                                iter.next();
                            }
                            if !string_closed {
                                panic!("Expected \"");
                            }
                            cnt += 1; 
                            Token::String(s) 
                        },
                        '-' | '0'..='9' => {
                            let mut s = String::new();
                            s.push(*ch);
                            iter.next();
                            let mut is_floating = false;
                            while let Some((_, ch)) = iter.peek() {
                                if *ch >= '0' && *ch <= '9' || (!is_floating && *ch == '.') {
                                    s.push(*ch);
                                    if *ch == '.' {
                                        is_floating = true;
                                    }
                                    cnt += 1;
                                    iter.next();
                                } else {
                                    break;
                                }
                            }
                            let num = s.parse::<f32>();
                            if let Err(n) = num {
                                panic!("Parse error. String was {}", s);
                            }
                            Token::Number(num.unwrap())
                        },
                        x => Token::OtherChar(*x)
                    };
                    let is_num = match tok {
                        Token::Number(_) => true,
                        _ => false
                    };
                    self.tokens.push_back(LexicalToken{
                        token: tok,
                        from: start_loc,
                        to: SourceLocation {
                            row: lnum,
                            col: cnt
                        }
                    });
                    if !is_num {
                        iter.next();
                   }
                }
            }
        }

        fn parse_array(&mut self) -> Result<Vec<Datum>, ParseError> {
            if self.tokens.is_empty() || self.tokens.front().unwrap().token != Token::OpenSquare {
                panic!("Invalid syntax");
            }
            
            let mut ret = Vec::<Datum>::new();

            let mut cur_phase = ParsePhase::FindValue;

            self.tokens.pop_front();
            loop {
                let cur_tok = self.tokens.front();
                if cur_tok.is_none() {
                    return Err(ParseError::UnexpectedEof);
                }
                if let Some(tok) = cur_tok {
                    match cur_phase {
                        ParsePhase::FindValue => {
                            let mut val: Datum;
                            match tok.token {
                                Token::OpenCurly => val = Datum::Object(self.parse_impl()?),
                                Token::String(str) => val = Datum::Str(str),
                                Token::Boolean(b) => val = Datum::Boolean(b),
                                Token::Null => val = Datum::Null,
                                Token::Number(f) => val = Datum::Number(f),
                                _ => return Err(ParseError::UnexpectedToken(tok.clone()))
                            }
                            ret.push(val);
                            cur_phase = ParsePhase::FindCommaOrEnd;
                        },
                        ParsePhase::FindCommaOrEnd => {
                            match tok.token {
                                Token::CloseSquare => return Ok(ret),
                                Token::Comma => cur_phase = ParsePhase::FindValue,
                                _ => return Err(ParseError::UnexpectedToken(tok.clone()))
                            }
                        },
                        _ => panic!("Unexpected parse phase!")
                    }
                }
            
                self.tokens.pop_front();
            }
        }

        fn parse_impl(&mut self) -> Result<HashMap::<String, Datum>, ParseError> {
            if self.tokens.is_empty() || self.tokens.front().unwrap().token != Token::OpenCurly {
                panic!("Invalid syntax");
            }
            
            let mut ret = HashMap::<String, Datum>::new();

            let mut cur_phase = ParsePhase::FindKeyOrEnd;
            let mut cur_key: Option<String> = None;

            self.tokens.pop_front();
            loop {
                let cur_tok = self.tokens.front();
                if cur_tok.is_none() {
                    return Err(ParseError::UnexpectedEof);
                }
                let tok = *cur_tok.unwrap();
                match cur_phase {
                    ParsePhase::FindKeyOrEnd => {
                        match tok.token {
                            Token::CloseCurly => return Ok(ret),
                            Token::String(str) => {
                                cur_key = Some(str);
                                cur_phase = ParsePhase::FindColon;
                            },
                            _ => return Err(ParseError::UnexpectedToken(tok))
                        }
                    },
                    ParsePhase::FindKey => {
                        match tok.token {
                            Token::String(str) => {
                                cur_key = Some(str);
                                cur_phase = ParsePhase::FindColon;
                            },
                            _ => return Err(ParseError::UnexpectedToken(tok))
                        }
                    },
                    ParsePhase::FindColon => {
                        match tok.token {
                            Token::Colon => cur_phase = ParsePhase::FindValue,
                            _ => return Err(ParseError::UnexpectedToken(tok))
                        }
                    },
                    ParsePhase::FindValue => {
                        let mut val: Datum;
                        match tok.token {
                            Token::OpenCurly => val = Datum::Object(self.parse_impl()?),
                            Token::String(str) => val = Datum::Str(str),
                            Token::Boolean(b) => val = Datum::Boolean(b),
                            Token::Null => val = Datum::Null,
                            Token::Number(f) => val = Datum::Number(f),
                            _ => return Err(ParseError::UnexpectedToken(tok))
                        }
                        assert!(cur_key.is_some());
                        if let Some(key) = cur_key {
                            ret.insert(key, val);
                        }
                        cur_phase = ParsePhase::FindCommaOrEnd;
                    },
                    ParsePhase::FindCommaOrEnd => {
                        match tok.token {
                            Token::Comma => cur_phase = ParsePhase::FindKey,
                            Token::CloseCurly => return Ok(ret),
                            _ => return Err(ParseError::UnexpectedToken(tok))
                        }
                    }
                };

                self.tokens.pop_front();
            }
        }

        pub fn parse(&mut self, src: &str) -> bool {
            self.lex(src);
            dbg!(&self.tokens);


            let root = self.parse_impl();
            match root {
                Ok(r) => self.root = Some(r),
                Err(e) => println!("Error parsing string: {:?}", e)
            }
            return root.is_ok();
        }
    }
}

#[cfg(test)]
mod tests {

}
