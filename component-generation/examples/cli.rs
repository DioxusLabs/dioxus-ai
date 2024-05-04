use component_generation::generate_ui;
use kalosm::language::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[tokio::main]
async fn main() {
    loop {
        let input = prompt_input("What do you want to make? ").unwrap();
        let start_timestamp = std::time::Instant::now();
        let state = generate_ui(&input).await;

        let app = state.app_component();
        print_component(&app);

        for component in state.components() {
            print_component(&component.component_string());
        }

        println!("\nTook: {:?}", start_timestamp.elapsed());
    }
}

fn print_component(component: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(component) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }

    println!("\x1b[0m");
    println!("\n");
}
