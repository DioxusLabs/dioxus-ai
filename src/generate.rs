use core::panic;
use dioxus_autofmt::write_block_out;
use kalosm::language::*;
use rsx_rosetta::{rsx_from_html, Dom};
use std::io::Write;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

const REGEX_CONSTRAINTS: &str = r#"[^\n]+\nCOMPONENTS:\n(- [A-Z][a-z]\w+: [\w\d\.\- ]+\n)+HTML:\n[^\n]+\nCOMPONENT HTML:(\n[A-Z][a-z]\w+:\n[^\n]+)+<\|end_of_text\|>"#;

pub async fn generate() {
    let model = FileSource::huggingface(
        "Demonthos/llama3".to_string(),
        "main".to_string(),
        "llama3-unsloth.Q4_K_M.gguf".to_string(),
    );
    let tokenizer = FileSource::huggingface(
        "NousResearch/Meta-Llama-3-8B".to_string(),
        "main".to_string(),
        "tokenizer.json".to_string(),
    );
    let llm = Llama::builder()
        .with_source(LlamaSource::new(model, tokenizer))
        .build()
        .await
        .unwrap();

    loop {
        let constraints = RegexParser::new(REGEX_CONSTRAINTS).unwrap();
        let input = prompt_input("What do you want to make? ").unwrap();
        let prompt = input + "\nDESCRIPTION:\n";
        let start_timestamp = std::time::Instant::now();
        let mut stream = llm
            .stream_structured_text(&prompt, constraints)
            .await
            .unwrap();

        let mut state = PartialState::new();

        let mut current_line = String::new();
        while let Some(text) = stream.next().await {
            current_line.push_str(&text);
            let lines = current_line.lines().count();
            if lines > 1 {
                let mut lines_iter = current_line.lines();
                for line in (&mut lines_iter).take(lines - 1) {
                    state.process_line(line);
                }
                current_line = lines_iter.next().unwrap().to_string();
            }
        }
        state.process_line(&current_line);
        state.next_section();

        println!("\nTook: {:?}", start_timestamp.elapsed());
    }
}

#[derive(Debug)]
struct PartialState {
    current_section: Section,
    description: String,
    html: String,
    current_component_index: Option<usize>,
    components: Vec<Component>,
}

impl PartialState {
    fn new() -> Self {
        Self {
            current_section: Section::Description,
            description: String::new(),
            html: String::new(),
            current_component_index: None,
            components: Vec::new(),
        }
    }

    fn next_section(&mut self) {
        // Finish the current section
        match self.current_section {
            Section::HTML => {
                print!("HTML:");
                let nodes = Dom::parse(&self.html).unwrap();
                let rsx = rsx_from_html(&nodes);
                if let Some(block) = write_block_out(rsx) {
                    print_rsx(&block);
                }
            }
            Section::ComponentHTML => {
                for component in self.components.iter() {
                    print!("{} ({})", component.name, component.description);
                    let nodes = Dom::parse(&component.html).unwrap();
                    let rsx = rsx_from_html(&nodes);
                    if let Some(block) = write_block_out(rsx) {
                        print_rsx(&block);
                    }
                }
            }
            Section::Description => {
                println!("\nThinking about UI...");
            }
            Section::Components => {}
        };

        if let Some(next_section) = self.current_section.next_section() {
            #[allow(clippy::single_match)]
            match next_section {
                Section::Components => {
                    println!("I think I will need components for this...");
                }
                _ => {}
            }

            self.current_section = next_section;
        }
    }

    fn process_line(&mut self, line: &str) {
        if line.trim().is_empty() {
            return;
        }

        if let Some(next_section) = self.current_section.next_section() {
            if line.to_lowercase().trim().replace([':', '-'], "")
                == next_section.identifier().to_lowercase()
            {
                self.next_section();

                return;
            }
        }

        match self.current_section {
            Section::Description => {
                self.description.push_str(line);
                print!("{}", line);
                std::io::stdout().flush().unwrap();
            }
            Section::Components => {
                let (before_colon, after_colon) = line.trim().split_once(':').unwrap();
                let name = before_colon
                    .strip_prefix('-')
                    .unwrap_or(before_colon)
                    .trim()
                    .to_string();
                let description = after_colon.trim().to_string();
                let component = Component {
                    name,
                    description,
                    html: String::new(),
                };
                println!("- {} ({})", component.name, component.description);
                self.components.push(component);
            }
            Section::HTML => {
                self.html.push_str(line.trim());
            }
            Section::ComponentHTML => {
                let html = line.trim().to_string();
                match self.current_component_index {
                    Some(index) => {
                        // Only keep the HTML up to the end of the text
                        let html = html.split_once("<|end_of_text|>").map(|x| x.0).unwrap_or(&html);
                        self.components[index].html = html.to_string();
                        self.current_component_index = None;
                    }
                    None => {
                        let trimmed_line = line
                            .chars()
                            .filter(|c| c.is_ascii_alphabetic())
                            .collect::<String>()
                            .to_lowercase();
                        let index = self
                            .components
                            .iter()
                            .position(|x| x.name.to_lowercase().trim() == trimmed_line)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Component {} not found in {:?}",
                                    line,
                                    self.components
                                        .iter()
                                        .map(|x| x.name.to_lowercase())
                                        .collect::<Vec<String>>()
                                )
                            });
                        println!("Creating HTML for {}...", self.components[index].name);
                        self.current_component_index = Some(index);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Component {
    name: String,
    description: String,
    html: String,
}

#[derive(Debug)]
enum Section {
    Description,
    Components,
    #[allow(clippy::upper_case_acronyms)]
    HTML,
    ComponentHTML,
}

impl Section {
    fn identifier(&self) -> &str {
        match self {
            Section::Description => "DESCRIPTION",
            Section::Components => "COMPONENTS",
            Section::HTML => "HTML",
            Section::ComponentHTML => "COMPONENT HTML",
        }
    }

    fn next_section(&self) -> Option<Section> {
        match self {
            Section::Description => Some(Section::Components),
            Section::Components => Some(Section::HTML),
            Section::HTML => Some(Section::ComponentHTML),
            Section::ComponentHTML => None,
        }
    }
}

fn print_rsx(rsx: &str) {
    // First trim the indentation
    let rsx = rsx
        .lines()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<&str>>()
        .join("\n");

    println!("\n");

    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(&rsx) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
    println!("\x1b[0m");
    println!("\n");
}
