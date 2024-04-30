// TODO: It might be better to fork html5ever for more robust parsing
// This parser is very loose. It will parse text that is not valid HTML, but it is fast
// HTML is not a regular language, so it is not possible to correctly parse it with a regular expression

use const_format::concatcp;

const ELEMENT_NAME_REGEX: &str = r#"[a-z]+"#;
const ATTRIBUTE_REGEX: &str = r#" [a-zA-Z]+="[^"]+""#;
const ELEMENT_START_REGEX: &str = concatcp!(
    "(",
    "<",
    ELEMENT_NAME_REGEX,
    "(",
    ATTRIBUTE_REGEX,
    ")*",
    ">",
    ")"
);
const CLOSE_TAG_REGEX: &str = concatcp!("(</", ELEMENT_NAME_REGEX, ">)");

const TEXT_REGEX: &str = r#"([^<]+)"#;

const HTML_REGEX: &str = concatcp!(
    "(",
    ELEMENT_START_REGEX,
    "|",
    CLOSE_TAG_REGEX,
    "|",
    TEXT_REGEX,
    ")+"
);

#[test]
fn test_regex() {
    println!("{}", HTML_REGEX);

    let regex_parser = RegexParser::new(HTML_REGEX).unwrap();

    let state = regex_parser.create_parser_state();
    let result = regex_parser.parse(&state, b"<div></div>");
    assert!(result.is_ok());

    let state = regex_parser.create_parser_state();
    let result = regex_parser.parse(&state, b"<Children>");
    assert!(result.is_err());
}
