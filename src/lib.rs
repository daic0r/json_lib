pub mod json_lib2 {
    use std::collections::HashMap;

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
        Object(HashMap<String, Datum>),
        Array(Vec<Datum>)
    }

    #[derive(Debug, Clone)]
    struct SourceLocation {
        col: usize,
        row: usize
    }

    #[derive(Debug)]
    struct LexicalToken {
        token: Token,
        from: SourceLocation,
        to: SourceLocation
    }

    pub struct Json {
        //root: HashMap<String, Datum>,
        tokens: Vec<LexicalToken>,
        tok_expected: Vec<Token>
    }

    impl Json {
        pub fn new() -> Self {
            Json {
                tokens: vec![],
                tok_expected: vec![Token::OpenCurly]
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
                    self.tokens.push(LexicalToken{
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

        pub fn parse(&mut self, src: &str) {
            self.lex(src);
            dbg!(&self.tokens);
        }
    }
}

#[cfg(test)]
mod tests {

}
