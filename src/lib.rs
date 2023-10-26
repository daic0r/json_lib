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

    #[derive(Debug, PartialEq)]
    pub enum Datum {
        Str(String),
        Number(f32),
        Boolean(bool),
        Null,
        Object(HashMap::<String, Datum>),
        Array(Vec<Datum>)
    }

    #[derive(Debug, Clone)]
    pub struct SourceLocation {
        col: usize,
        row: usize
    }

    #[derive(Debug, Clone)]
    pub struct LexicalToken {
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
            let mut ret = Vec::<Datum>::new();

            let mut cur_phase = ParsePhase::FindValue;

            loop {
                let cur_tok = self.tokens.pop_front();
                if cur_tok.is_none() {
                    return Err(ParseError::UnexpectedEof);
                }
                if let Some(tok) = cur_tok {
                    match cur_phase {
                        ParsePhase::FindValue => {
                            let val: Datum;
                            match tok.token {
                                Token::OpenCurly => val = Datum::Object(self.parse_impl()?),
                                Token::OpenSquare => val = Datum::Array(self.parse_array()?),
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
            }
        }

        fn parse_impl(&mut self) -> Result<HashMap::<String, Datum>, ParseError> {
            let mut ret = HashMap::<String, Datum>::new();

            let mut cur_phase = ParsePhase::FindKeyOrEnd;
            let mut cur_key: Option<String> = None;

            loop {
                let cur_tok = self.tokens.pop_front();
                if cur_tok.is_none() {
                    return Err(ParseError::UnexpectedEof);
                }
                if let Some(tok) = cur_tok {
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
                            let val: Datum;
                            match tok.token {
                                Token::OpenCurly => val = Datum::Object(self.parse_impl()?),
                                Token::OpenSquare => val = Datum::Array(self.parse_array()?),
                                Token::String(str) => val = Datum::Str(str),
                                Token::Boolean(b) => val = Datum::Boolean(b),
                                Token::Null => val = Datum::Null,
                                Token::Number(f) => val = Datum::Number(f),
                                _ => return Err(ParseError::UnexpectedToken(tok))
                            }
                            assert!(cur_key.is_some());
                            if let Some(ref key) = cur_key {
                                ret.insert(key.to_string(), val);
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
                }
            }
        }

        pub fn parse(&mut self, src: &str) -> bool {
            self.lex(src);
            dbg!(&self.tokens);


            if self.tokens.is_empty() || self.tokens.pop_front().unwrap().token != Token::OpenCurly {
                panic!("Invalid syntax");
            }
            
            let root = self.parse_impl();
            match root {
                Ok(r) => {
                    assert!(self.tokens.is_empty());
                    self.root = Some(r);
                    return true;
                },
                Err(e) => println!("Error parsing string: {:?}", e)
            }
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parser() {
        use crate::json_lib2::Json;
        let mut json = Json::new();
        let src = r#"
        {
            "name": "John",
            "age": 30,
            "cars": {
                "car1": "Ford",
                "car2": "BMW",
                "car3": "Fiat"
            },
            "children": [
                "Ann",
                "Billy"
            ]
        }
        "#;
        let res = json.parse(src);
        assert!(res);
        
        let root = json.root.unwrap();
        assert_eq!(root["name"], crate::json_lib2::Datum::Str(String::from("John")));
        assert_eq!(root["age"], crate::json_lib2::Datum::Number(30.0));
        assert_eq!(root["cars"], crate::json_lib2::Datum::Object(
            vec![
                (String::from("car1"), crate::json_lib2::Datum::Str(String::from("Ford"))),
                (String::from("car2"), crate::json_lib2::Datum::Str(String::from("BMW"))),
                (String::from("car3"), crate::json_lib2::Datum::Str(String::from("Fiat")))
            ].into_iter().collect()
        ));
        assert_eq!(root["children"], crate::json_lib2::Datum::Array(
            vec![
                crate::json_lib2::Datum::Str(String::from("Ann")),
                crate::json_lib2::Datum::Str(String::from("Billy"))
            ]
        ));
    }

}
