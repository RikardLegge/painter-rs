use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;

fn main() {
    let path = Path::new("example.css");
    let file_path_display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", file_path_display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();
    let file_contents = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", file_path_display,
                           why.description()),
        Ok(_) => s,
    };

    let mut parser = CssParser::new();
    parser.parse(file_contents);
}


trait Css {
    fn test(char : char) -> CssContext;
    fn begin(state : &mut CssParser) {}
    fn append(state : &mut CssParser) {}
    fn end(state : &mut CssParser) {}
}

struct CssNull {}
impl Css for CssNull {
    fn test(char : char) -> CssContext { return CssContext::Unknown; }
}

struct CssString {}
impl Css for CssString {
    fn test(css : char) -> CssContext {
        match css {
            '"' => CssContext::End,
            '\'' => CssContext::End,
            _ => CssContext::String,
        }
    }

    fn end(state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }
}

struct CssValue {}
impl Css for CssValue {
    fn test(css : char) -> CssContext {
        match css {
            '"' => CssContext::String,
            '\'' => CssContext::String,
            ';' => CssContext::End,
            '}' => CssContext::EndKeepChar,
            _ => CssContext::Value,
        }
    }

    fn begin(state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(state : &mut CssParser) {
        state.current_rule.value = state.flush_char_buffer();

        let current_rule = mem::replace(&mut state.current_rule, CssRule::new());
        state.current_rule_set.rules.push(current_rule);
    }
}

struct CssKey {}
impl Css for  CssKey {
    fn test(css : char) -> CssContext {
        match css {
            ':' => CssContext::End,
            _ => CssContext::Key,
        }
    }

    fn begin(state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(state : &mut CssParser) {
        state.current_rule.key = state.flush_char_buffer();
        state.push_context(CssContext::Value);
    }
}

struct CssSelector {}
impl Css for CssSelector {
    fn test(css : char) -> CssContext {
        match css {
            '{' => CssContext::End,
            ',' => CssContext::Append,
            _ => CssContext::Selector,
        }
    }

    fn begin(state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(state : &mut CssParser) {
        let chars = state.flush_char_buffer();
        state.current_rule_set.selectors.push(chars);

        state.push_context(CssContext::RuleSet);
    }

    fn append(state : &mut CssParser) {
        let chars = state.flush_char_buffer();
        state.current_rule_set.selectors.push(chars);
    }
}

impl Css for CssRuleSet {
    fn test(css : char) -> CssContext {
        match css {
            ' ' => CssContext::RuleSet,
            '\n' => CssContext::RuleSet,
            '\r' => CssContext::RuleSet,
            '\t' => CssContext::RuleSet,
            '}' => CssContext::End,
            _ => CssContext::Key
        }
    }

    fn end(state : &mut CssParser) {
        let current_rule_set = mem::replace(&mut state.current_rule_set, CssRuleSet::new());
        state.rule_sets.push(current_rule_set);

    }
}

struct CssRoot {
    rule_sets : Vec<CssRuleSet>,
}
impl CssRoot {
    fn new() -> CssRoot {
        return CssRoot {
            rule_sets: Vec::new()
        }
    }
}
impl Css for CssRoot {
    fn test(css : char) -> CssContext {
        match css {
            _ => CssContext::Selector,
        }
    }

    fn begin(state : &mut CssParser) {
//        return CssRoot::new();
    }
}

struct CssParser {
    stack : Vec<CssContext>,
    char_buffer : Vec<char>,
    current_char: char,

    rule_sets : Vec<CssRuleSet>,
    current_rule_set : CssRuleSet,
    current_rule : CssRule
}

impl CssParser {

    fn new() -> CssParser {
        let mut parser = CssParser {
            stack: Vec::new(),
            char_buffer: Vec::new(),
            current_char: '\0',

            rule_sets: Vec::new(),
            current_rule_set: CssRuleSet::new(),
            current_rule: CssRule::new()
        };
        parser.push_context(CssContext::Root);

        return parser;
    }

    fn push_context(&mut self, context : CssContext) {
        self.stack.push(context);
    }

    fn pop_context(&mut self) {
        let _ = self.stack.pop();
    }

    fn flush_char_buffer(&mut self) -> String {
        let val: String = self.char_buffer.iter().cloned().collect();
        self.char_buffer.clear();
        return val.trim().to_string();
    }

    fn push_char(&mut self, char: char) {
        self.char_buffer.push(char);
    }

    fn parse_char(&mut self) {
        let char = self.current_char;
        let current_context = match self.stack.last() {
            Some(x) => *x,
            None => CssContext::Unknown
        };

        let next_context = match current_context {
            CssContext::Root => CssRoot::test(char),
            CssContext::Selector => CssSelector::test(char),
            CssContext::RuleSet => CssRuleSet::test(char),
            CssContext::Key => CssKey::test(char),
            CssContext::Value => CssValue::test(char),
            CssContext::String => CssString::test(char),
            x => panic!("{:?}", x)
        };

        match next_context {
            CssContext::End | CssContext::EndKeepChar => {
                self.pop_context();

                match current_context {
                    CssContext::Selector => { CssSelector::end(self) },
                    CssContext::Key => { CssKey::end(self) }
                    CssContext::Value => { CssValue::end(self) },
                    CssContext::RuleSet => { CssRuleSet::end(self) },
                    CssContext::String => { CssString::end(self) },
                    _ => ()
                };

                match next_context {
                    CssContext::EndKeepChar => { self.parse_char(); },
                    _ => ()
                };

            },
            CssContext::Append => {
                match current_context {
                    CssContext::Selector => { CssSelector::append(self) },
                    _ => ()
                }
            },
            ctx if current_context != next_context => {
                self.push_context(ctx);

                match ctx {
                    CssContext::Selector => { CssSelector::begin(self) },
                    CssContext::Key => { CssSelector::begin(self) },
                    CssContext::String => { CssSelector::begin(self) },
                    _ => ()
                }
            },
            _ => { self.push_char(char) }

        }
    }

    fn parse(&mut self, css : String) {
        for char in css.chars() {
            self.current_char = char;
            self.parse_char();
        }

        println!("{:?}", self.rule_sets);
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum CssContext {
    Root,
    Selector,
    RuleSet,
    Key,
    Value,
    String,

    Append,
    End,
    EndKeepChar,
    Unknown,
}

#[derive(Debug)]
struct CssRule {
    key: String,
    value: String
}

impl CssRule {
    fn new() -> CssRule {
        return CssRule {key: "".to_string(), value: "".to_string()}
    }
}

#[derive(Debug)]
struct CssRuleSet {
    selectors : Vec<String>,
    rules : Vec<CssRule>
}

impl CssRuleSet {
    fn new() -> CssRuleSet {
        return CssRuleSet {selectors: Vec::new(), rules: Vec::new()}
    }
}

