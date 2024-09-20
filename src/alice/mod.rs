use std::collections::HashMap;

// type Attributes = HashMap<String, String>;
#[derive(Debug, Clone)]
struct Attribute {
    key: String,
    value: String
}

type Attributes = Vec<Attribute>;

macro_rules! eof_reached {
    ($msg:expr) => {
        println!("{:?}", $msg);
        break;
    };
}

macro_rules! unknown_state_reached {
    ($last_state:expr) => {
        println!("unknown state encountered: {:?}", $last_state);
        break;
    };
}

/// In the context of the official html spec, next_input_character refers the next character in the
/// input stream which we can read,but here, next_input_character refers to the character
/// immediately after the current_input_character. In most cases, this distinction isn't important,
/// but it becomes important once we start getting into reconsuming characters
///

pub struct HTMLTokenizer {
    reconsume: bool,
    current_input_character: Option<char>,
    next_input_character: Option<char>,
    state: HTMLTokenizerState,
    return_state: Option<HTMLTokenizerState>, // None by default
    input_stream: Vec<char>,
    cursor: Option<usize>,
    tokens: Vec<HTMLToken>,
    current_doctype_token: Option<DOCTYPE>,
    current_tag_token: Option<Tag>,
    current_comment_token: Option<Comment>,
}

#[derive(Debug)]
pub enum HTMLToken {
    Doctype(DOCTYPE),
    Tag(Tag),
    Comment(Comment),
    Character(Character),
    EndOfFile,
}

#[derive(Clone, Debug)]
pub struct DOCTYPE {
    name: Option<String>,              // None by default which is different from ""
    public_identifier: Option<String>, // None by default which is different from ""
    system_identifier: Option<String>, // None by default which is different from ""
    force_quirks: bool,                // false by default
}

#[derive(Clone, Debug)]
pub enum Tag {
    StartTag {
        tag_name: String,
        self_closing: bool,     // false by default
        attributes: Attributes, // empty by default
    },
    EndTag {
        tag_name: String,
        self_closing: bool,     // false by default
        attributes: Attributes, // empty by default
    },
}

#[derive(Clone, Debug)]
pub struct Comment {
    data: String,
}

#[derive(Debug)]
pub struct Character {
    data: String,
}

impl HTMLTokenizer {
    pub fn new(input_stream: &Vec<char>) -> Self {
        Self {
            state: HTMLTokenizerState::Data,
            reconsume: false,
            return_state: None,
            input_stream: input_stream.clone(),
            cursor: Some(0),
            tokens: Vec::new(),
            current_input_character: None,
            next_input_character: input_stream.clone().get(0).copied(), // char at index 0;
            current_doctype_token: None,
            current_tag_token: None,
            current_comment_token: None,
        }
    }

    fn consume_next_input_character(&mut self) -> &Option<char> {
        if self.reconsume {
            println!("reconsume @ {:?}", self.state);
            println!(
                "c_i_c and n_i_c : {:?} {:?}",
                self.current_input_character, self.next_input_character
            );
            // Dont advance the cursor. Just return current_input_character as is.

            // reset self.reconsume
            self.reconsume = false;
        } else {
            // set current_input_character to whatever the value of next_input_character was
            self.current_input_character = self.next_input_character.clone();
            // advance the cursor by 1
            match self.cursor {
                None => self.cursor = Some(0),
                Some(value) => self.cursor = Some(value + 1),
            };
            // then next_input_character should be the next character in the input stream
            self.next_input_character = self.input_stream.get(self.cursor.unwrap()).copied();
        }
        // return a reference to the current input character
        return &self.current_input_character;
    }

    fn switch_state(&mut self, state: HTMLTokenizerState) {
        self.state = state;
    }

    pub fn run(&mut self) {
        loop {
            // print the tokens
            // println!("tokens: {:?}", self.tokens);

            // ... iterating html string
            // checking for eof
            // println!("char:{:?}", self.current_input_character);
            // if self.current_input_character == None && self.next_input_character == None {
            //     break;
            // }
            // self.consume_next_input_character();
            // continue;
            // ...

            match self.state {
                // Data state
                HTMLTokenizerState::Data => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '&' => {}
                            '<' => {
                                self.switch_state(HTMLTokenizerState::TagOpen);
                            }
                            '\0' => {}
                            _ => {
                                // emit current_input_character as a character token
                                let character_token = Character {
                                    data: current_input_character.clone().to_string(),
                                };
                                self.emit_token(HTMLToken::Character(character_token));
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Tag Open state
                HTMLTokenizerState::TagOpen => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '!' => self.switch_state(HTMLTokenizerState::MarkupDeclarationOpen),
                            '/' => self.switch_state(HTMLTokenizerState::EndTagOpen),
                            'a'..='z' | 'A'..='Z' => {
                                // create a start tag token
                                self.current_tag_token = Some(Tag::StartTag {
                                    tag_name: "".to_string(),
                                    self_closing: false,
                                    attributes: Vec::new()
                                });
                                // reconsume the current_input_character in the Tag Name state
                                self.switch_state(HTMLTokenizerState::TagName);
                                self.reconsume = true;
                            }
                            _ => {
                                // just ignore it
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // End Tag Open state
                HTMLTokenizerState::EndTagOpen => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            'a'..='z' | 'A'..='Z' => {
                                // create a end tag token, set it's tag_name value to empty string
                                self.current_tag_token = Some(Tag::EndTag {
                                    tag_name: "".to_string(),
                                    self_closing: false,
                                    attributes: Vec::new()
                                });

                                // reconsume current_input_character in the tag name state
                                self.switch_state(HTMLTokenizerState::TagName);
                                self.reconsume = true;
                            }
                            _ => {
                                // ignore for now
                            }
                        }
                    }
                }

                // Tag Name state
                HTMLTokenizerState::TagName => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                self.switch_state(HTMLTokenizerState::BeforeAttributeName);
                            }
                            '/' => {
                                // ignore for now
                            }
                            '>' => {
                                // switch to data state
                                self.switch_state(HTMLTokenizerState::Data);
                                //emit current tag token.
                                self.emit_current_tag_token();
                            }
                            'A'..='Z' => {
                                // ignore for now
                            }
                            '\0' => {
                                // ignore for now
                            }
                            _ => {
                                // append current_input_character to current tag token's tag name
                                if let Some(tag_token) = &mut self.current_tag_token {
                                    // append current_input_character to start tag's name
                                    match tag_token {
                                        Tag::StartTag {
                                            tag_name,
                                            self_closing: _,
                                            attributes: _,
                                        }
                                        | Tag::EndTag {
                                            tag_name,
                                            self_closing: _,
                                            attributes: _,
                                        } => {
                                            tag_name.push(current_input_character);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Before Attribute Name state
                HTMLTokenizerState::BeforeAttributeName => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                // ignore the character
                                continue;
                            }
                            '/' | '>'   // also matches for eof
                            => {
                                // ignore for now
                            }
                            '=' => {}
                            _ => {
                                // start a new attribute in the current tag token. Set that
                                // attribute name and value to empty string.
                                if let Some(tag_token) = &mut self.current_tag_token {
                                    match tag_token {
                                        Tag::StartTag {
                                            tag_name: _,
                                            self_closing: _,
                                            attributes,
                                        }
                                        | Tag::EndTag {
                                            tag_name: _,
                                            self_closing: _,
                                            attributes,
                                        } => {
                                            // push a new attribute to the attributes vector
                                            let new_attribute = Attribute{key: "".to_string(), value: "".to_string()};
                                            attributes.push(new_attribute);
                                            // switch to Attribute name state and reconsume the
                                            // current_input_character
                                            self.switch_state(HTMLTokenizerState::AttributeName);
                                            self.reconsume = true;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Attribute Name state
                HTMLTokenizerState::AttributeName => {
                    if let Some(current_input_character) = self.consume_next_input_character().clone() {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' | '/' | '>'   // also includes an eof
                            => {
                                // ignore for now
                            }
                            '=' => {
                                self.switch_state(HTMLTokenizerState::BeforeAttributeValue);
                            }
                            'A'..='Z' => {}
                            '\0' => {}
                            '"' | '\'' | '<' => {}
                            _ => {
                                // append current_input_character to current attribute's name
                                if let Some(tag_token) = &mut self.current_tag_token {
                                    match tag_token{
                                        Tag::StartTag { tag_name: _, self_closing: _, attributes } 
                                        | 
                                        Tag::EndTag { tag_name: _, self_closing: _, attributes } => {
                                            // the `current attribute` is the last attribute in the
                                            // attributes vector
                                            if let Some(current_attribute) = attributes.last_mut(){
                                                current_attribute.key.push(current_input_character);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }


                // Before Attribute Value state
                HTMLTokenizerState::BeforeAttributeValue => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                // ignore the character
                                continue;
                            }
                            '"' => {
                                self.switch_state(HTMLTokenizerState::AttributeValueDoubleQuoted);
                            }
                            '\'' => {
                                self.switch_state(HTMLTokenizerState::AttributeValueSingleQuoted);
                            }
                            '>' => {}
                            _ => {
                                // ignore for now
                            }
                        }
                    }
                }

                // Attribute Value Single Quoted state
                HTMLTokenizerState::AttributeValueSingleQuoted => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '\'' => self.switch_state(HTMLTokenizerState::AfterAttributeValueQuoted),
                            '&' => {}
                            '\0' => {}
                            _=> {
                                self.append_current_input_character_to_current_attribute_value();
                            }
                        }
                    }
                }


                // Attribute Value Double Quoted state
                HTMLTokenizerState::AttributeValueDoubleQuoted => {
                    if let Some(current_input_character) = self.consume_next_input_character().clone() {
                        match current_input_character {
                            '"' => {
                                self.switch_state(HTMLTokenizerState::AfterAttributeValueQuoted);
                            }
                            '&' => {}
                            '\0' => {}
                            _=> {
                                self.append_current_input_character_to_current_attribute_value();
                            }
                        }
                    }
                }

                // After Attribute Value Quoted state
                HTMLTokenizerState::AfterAttributeValueQuoted => {
                    if let Some(current_input_character) = self.consume_next_input_character(){
                        match current_input_character{
                            '\t' | '\n' | '\x0C' | ' ' => {
                                self.switch_state(HTMLTokenizerState::BeforeAttributeName);
                            }
                            '/' => {
                                // ignore for now
                            }
                            '>' => {
                                // emit current_tag_token
                                self.emit_current_tag_token();
                                // switch to data state
                                self.switch_state(HTMLTokenizerState::Data);
                            }
                            _=> {
                                // ignore for now
                            }
                        }
                    }
                }

                // Markup Declaration Open state
                HTMLTokenizerState::MarkupDeclarationOpen => {
                    if self.next_few_characters_are("DOCTYPE".to_string()) {
                        // consume/move cursor over "DOCTYPE"
                        self.consume_substring("DOCTYPE".to_string());
                        self.switch_state(HTMLTokenizerState::DOCTYPE);
                    } else if self.next_few_characters_are("--".to_string()) {
                        // consume "--"
                        self.consume_substring("--".to_string());
                        self.current_comment_token = Some(Comment {
                            data: "".to_string(),
                        });
                        self.switch_state(HTMLTokenizerState::CommentStart);
                    } else {
                        // ignore it
                    }
                }

                // Comment Start state
                HTMLTokenizerState::CommentStart => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '-' => {}
                            '>' => {}
                            _ => {
                                // reconsume in the comment state
                                self.switch_state(HTMLTokenizerState::Comment);
                                self.reconsume = true;
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Comment state
                HTMLTokenizerState::Comment => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '<' => {}
                            '-' => self.switch_state(HTMLTokenizerState::CommentEndDash),
                            '\0' => {}
                            _ => {
                                // append the current_input_character to the existing comment
                                // token's data
                                if let Some(comment_token) = &mut self.current_comment_token {
                                    comment_token.data.push(current_input_character);
                                }
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Comment End Dash state
                HTMLTokenizerState::CommentEndDash => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '-' => self.switch_state(HTMLTokenizerState::CommentEnd),
                            _ => {}
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Comment End state
                HTMLTokenizerState::CommentEnd => {
                    if let Some(current_input_character) =
                        self.consume_next_input_character().clone()
                    {
                        match current_input_character {
                            '>' => {
                                //
                                self.switch_state(HTMLTokenizerState::Data);
                                // emit the current comment token.
                                self.emit_current_comment_token();
                            }
                            '!' => {}
                            '-' => {}
                            _ => {}
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // DOCTYPE state
                HTMLTokenizerState::DOCTYPE => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            ' ' | '\t' | '\n' | '\x0C' => {
                                self.switch_state(HTMLTokenizerState::BeforeDOCTYPEName)
                            }
                            '>' => {}
                            _ => {
                                // just ignore it
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // Before DOCTYPE Name state
                HTMLTokenizerState::BeforeDOCTYPEName => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            ' ' | '\t' | '\n' | '\x0C' => {
                                // ignore the character which probably means don't do anything?
                                continue;
                            }
                            'A'..='Z' => {}
                            '\0' => {
                                panic!("encountered a null character");
                            }
                            '>' => {}
                            _ => {
                                // create a new DOCTYPE token
                                self.current_doctype_token = Some(DOCTYPE {
                                    name: Some(
                                        self.current_input_character.unwrap().to_string().clone(),
                                    ),
                                    force_quirks: false,
                                    public_identifier: None,
                                    system_identifier: None,
                                });
                                // switch to DOCTYPE name state
                                self.switch_state(HTMLTokenizerState::DOCTYPEName);
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // DOCTYPE Name state
                HTMLTokenizerState::DOCTYPEName => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                self.switch_state(HTMLTokenizerState::AfterDOCTYPEName);
                            }
                            '>' => {
                                self.switch_state(HTMLTokenizerState::Data);
                                // emit current doctype token
                                self.emit_current_doctype_token();
                            }
                            'A'..='Z' => {
                                // ignore for now
                            }

                            '\0' => {
                                // ignore for now
                            }

                            _ => {
                                // append current_input_character to current_doctype_token's name
                                if let Some(doctype_token) = &mut self.current_doctype_token {
                                    if let Some(name) = &mut doctype_token.name {
                                        name.push(self.current_input_character.unwrap().clone());

                                        println!("name: {}", name);
                                    }
                                }
                            }
                        }
                    } else {
                        eof_reached!("end of file reached");
                    }
                }

                // After DOCTYPE Name state
                HTMLTokenizerState::AfterDOCTYPEName => {
                    if let Some(current_input_character) = self.consume_next_input_character() {
                        match current_input_character {
                            '\t' | '\n' | '\x0C' | ' ' => {
                                // ignore the character;
                                continue;
                            }
                            '>' => {
                                self.switch_state(HTMLTokenizerState::Data);
                                self.emit_current_doctype_token();
                            }
                            _ => {}
                        }
                    }
                }

                // catch any unimplemented state and stop the tokenization process
                _ => {
                    unknown_state_reached!(self.state);
                }
            }
        }

        // print the tokens
        println!("tokens: {:#?}", self.tokens);
    }

    fn append_current_input_character_to_current_attribute_value(&mut self){
        if let Some(tag_token) = &mut self.current_tag_token{
            match tag_token{
                Tag::StartTag { tag_name: _, self_closing: _, attributes }
                |
                Tag::EndTag { tag_name: _, self_closing: _, attributes } =>  {
                    if let Some(current_attribute) = attributes.last_mut(){
                        current_attribute.value.push(self.current_input_character.clone().unwrap());
                    }
                }
            }
        }
    }

    fn emit_token(&mut self, token: HTMLToken) {
        self.tokens.push(token);
    }

    fn emit_current_tag_token(&mut self) {
        if let Some(tag_token) = self.current_tag_token.clone() {
            self.emit_token(HTMLToken::Tag(tag_token));
        }
        // clear the current start tag token
        self.current_tag_token = None;
    }

    fn emit_current_comment_token(&mut self) {
        if let Some(comment_token) = self.current_comment_token.clone() {
            self.emit_token(HTMLToken::Comment(comment_token));
        }
        // clear the current comment token
        self.current_comment_token = None;
    }

    fn emit_current_doctype_token(&mut self) {
        if let Some(doctype_token) = self.current_doctype_token.clone() {
            self.emit_token(HTMLToken::Doctype(doctype_token));
        }
        // clear the current doctype token
        self.current_doctype_token = None;
    }

    fn consume_substring(&mut self, substring: String) {
        // move the cursor forward by the length of the substring
        for _ in 0..substring.len() {
            self.consume_next_input_character();
        }
    }

    fn next_few_characters_are(&self, substring: String) -> bool {
        let mut start_index = self.cursor.unwrap().clone();
        for char in substring.chars() {
            if let Some(char_from_input_stream) = self.input_stream.get(start_index) {
                // case-insensitive comparison
                if &char_from_input_stream.to_lowercase().to_string()
                    != &char.to_lowercase().to_string()
                {
                    return false;
                }
                start_index += 1;
            } else {
                panic!("reached end of file");
            }
        }
        return true;
    }
}

#[derive(Debug)]
pub enum HTMLTokenizerState {
    Data,
    RCDATA,
    RAWTEXT,
    ScriptData,
    PLAINTEXT,
    TagOpen,
    EndTagOpen,
    TagName,
    RCDATALessThanSign,
    RCDATAEndTagOpen,
    RCDATAEndTagName,
    RAWTEXTLessThanSign,
    RAWTEXTEndTagOpen,
    RAWTEXTEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    DOCTYPE,
    BeforeDOCTYPEName,
    DOCTYPEName,
    AfterDOCTYPEName,
    AfterDOCTYPEPublicKeyword,
    BeforeDOCTYPEPublicIdentifier,
    DOCTYPEPublicIdentifierDoubleQuoted,
    DOCTYPEPublicIdentifierSingleQuoted,
    AfterDOCTYPEPublicIdentifier,
    BetweenDOCTYPEPublicAndSystemIdentifiers,
    AfterDOCTYPESystemKeyword,
    BeforeDOCTYPESystemIdentifier,
    DOCTYPESystemIdentifierDoubleQuoted,
    DOCTYPESystemIdentifierSingleQuoted,
    AfterDOCTYPESystemIdentifier,
    BogusDOCTYPE,
    CDATASection,
    CDATASectionBracket,
    CDATASectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
}
