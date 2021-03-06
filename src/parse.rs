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
    fn test(&self, char : char) -> CssTestResult;
    fn begin(&self) {}
    fn append(&self) {}
    fn end(&self) {}
}

#[derive(Debug)]
struct CssNone { }
impl CssNone {
    fn new() -> CssNone {
        return CssNone {}
    }
}
impl Css for CssNone {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            _ => CssTestResult {context: CssContext::None, command: CssCommand::None},
        }
    }
}

#[derive(Debug)]
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
    fn test(&self, css : char) -> CssTestResult {
        match css {
            _ => CssTestResult {context: CssContext::Selector, command: CssCommand::Begin},
        }
    }
}

#[derive(Debug)]
struct CssSelector {}
impl CssSelector {
    fn new() -> CssSelector {
        return CssSelector {}
    }
}
impl Css for CssSelector {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            '{' => CssTestResult {context: CssContext::None,     command: CssCommand::End},
            ',' => CssTestResult {context: CssContext::Selector, command: CssCommand::Append},
            _ =>   CssTestResult {context: CssContext::Selector, command: CssCommand::None},

        }
    }

    fn begin(&self, state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(&self, state : &mut CssParser) {
        let chars = state.flush_char_buffer();
        state.ruleset.selectors.push(chars);

        state.push_context(CssContext::RuleSet);
    }

    fn append(&self, state : &mut CssParser) {
        let chars = state.flush_char_buffer();
        state.ruleset.selectors.push(chars);
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
impl Css for CssRuleSet {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            ' '  |
            '\n' |
            '\r' |
            '\t' => CssTestResult {context: CssContext::RuleSet, command: CssCommand::None},
            '}' =>  CssTestResult {context: CssContext::None,    command: CssCommand::End},
            _ =>    CssTestResult {context: CssContext::Key,     command: CssCommand::Begin},
        }
    }

    fn end(&self, state : &mut CssParser) {
        let current_rule_set = mem::replace(&mut state.ruleset, CssRuleSet::new());
        state.root.rule_sets.push(current_rule_set);
    }
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
struct CssKey {}
impl CssKey {
    fn new() -> CssKey {
        return CssKey {}
    }
}
impl Css for  CssKey {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            ':' => CssTestResult {context: CssContext::None, command: CssCommand::End},
            _ => CssTestResult   {context: CssContext::Key,  command: CssCommand::None},
        }
    }

    fn begin(&self, state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(&self, state : &mut CssParser) {
        state.rule.key = state.flush_char_buffer();
        state.push_context(CssContext::Value);
    }
}

#[derive(Debug)]
struct CssValue {}
impl CssValue {
    fn new() -> CssValue {
        return CssValue {}
    }
}
impl Css for CssValue {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            '"' => CssTestResult {context: CssContext::String, command: CssCommand::Begin},
            '\''=> CssTestResult {context: CssContext::String, command: CssCommand::Begin},
            ';' => CssTestResult {context: CssContext::None,   command: CssCommand::End},
            '}' => CssTestResult {context: CssContext::None,   command: CssCommand::EndKeepChar},
            _ => CssTestResult   {context: CssContext::Value,  command: CssCommand::None},
        }
    }

    fn begin(&self, state : &mut CssParser) {
        let char = state.current_char;
        state.push_char(char);
    }

    fn end(&self, state : &mut CssParser) {
        state.rule.value = state.flush_char_buffer();

        let current_rule = mem::replace(&mut state.rule, CssRule::new());
        state.ruleset.rules.push(current_rule);
    }
}

#[derive(Debug)]
struct CssString {}
impl CssString {
    fn new() -> CssString {
        return CssString {}
    }
}
impl Css for CssString {
    fn test(&self, css : char) -> CssTestResult {
        match css {
            '"' |
            '\'' => CssTestResult {context: CssContext::None,   command: CssCommand::EndIncludeChar},
            _ =>    CssTestResult {context: CssContext::String, command: CssCommand::None},
        }
    }
}

struct CssParser {
    stack : Vec<CssContext>,
    char_buffer : Vec<char>,
    current_char: char,

    root: CssRoot,
    selector: CssSelector,
    ruleset: CssRuleSet,
    rule: CssRule,
    key: CssKey,
    value: CssValue,
    string: CssString,
    none: CssNone
}
impl CssParser {

    fn new() -> CssParser {
        let mut parser = CssParser {
            stack: Vec::new(),
            char_buffer: Vec::new(),
            current_char: '\0',

            root: CssRoot::new(),
            selector: CssSelector::new(),
            ruleset: CssRuleSet::new(),
            rule: CssRule::new(),
            key: CssKey::new(),
            value: CssValue::new(),
            string: CssString::new(),
            none: CssNone::new()
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

    fn get_css_for_context(&self, context : CssContext) -> &Css {
        return match context {
            CssContext::Root => &self.root,
            CssContext::Selector => &self.selector,
            CssContext::RuleSet => &self.ruleset,
            CssContext::Key => &self.key,
            CssContext::Value => &self.value,
            CssContext::String => &self.string,
            CssContext::None => &self.none
        }
    }

    fn parse_char(&mut self) {
        let char = self.current_char;
        let current_context = match self.stack.last() {
            Some(x) => *x,
            None => CssContext::None
        };

        let css = self.get_css_for_context(current_context);
        let test_result = css.test(char);
        let command = test_result.command;
        let next_context = test_result.context;

        match command {
            CssCommand::End => {
                self.pop_context();
                css.end();
            },
            CssCommand::EndIncludeChar => {
                self.pop_context();
                css.end();
                self.push_char(char);
            },
            CssCommand::EndKeepChar => {
                self.pop_context();
                css.end();
                self.parse_char();
            },
            CssCommand::Append => {
                css.append();
            },
            CssCommand::Begin => {
                self.push_context(next_context);
                css.begin();
            },
            CssCommand::None => {
                self.push_char(char)
            }
        }
    }

    fn parse(&mut self, css : String) {
        for char in css.chars() {
            self.current_char = char;
            self.parse_char();
        }

        println!("{:?}", self.root);
    }
}

struct CssTestResult {
    command: CssCommand,
    context: CssContext
}

enum CssCommand {
    Begin,
    Append,
    End,
    EndIncludeChar,
    EndKeepChar,
    None,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum CssContext {
    Root,
    Selector,
    RuleSet,
    Key,
    Value,
    String,
    None,
}
