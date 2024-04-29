use std::io::Read;

use data_gen::Train;

fn main() {
    let mut deserialized = Vec::new();
    // Read all files in the data folder
    let files = std::fs::read_dir("data").unwrap();
    for file in files.flatten() {
        if file.path().is_file() && file.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let path = file.path();
            let file = std::fs::File::open(path).unwrap();
            let mut reader = std::io::BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents).unwrap();
            let new_deserialized = serde_json::from_str::<Vec<Train>>(&contents).unwrap();
            deserialized.extend(new_deserialized);
        }
    }

    println!("deserialized: {}", deserialized.len());

    let filtered = deserialized
        .into_iter()
        .filter_map(|x| {
            let content = &x.response.choices[0].message.content;

            let contains_svg = content.contains("<svg");
            if contains_svg {
                return None;
            }

            ParsedResponse::new(content)
        })
        .collect::<Vec<ParsedResponse>>();
    println!("filtered: {}", filtered.len());

    let mut file = std::fs::File::create("filtered.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &filtered).unwrap();

    let validated = filtered
        .into_iter()
        .filter_map(ValidatedResponse::from_parsed_response)
        .collect::<Vec<ValidatedResponse>>();
    println!("validated: {}", validated.len());

    let mut file = std::fs::File::create("validated.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &validated).unwrap();
}

const QUESTIONS: &[&str] = &[
    "what should the ui look like?",
    "what are the individual components that make up the ui?",
    "what does the html for the ui look like?",
    "what is the html for each component?",
];

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ValidatedResponse {
    description: String,
    html: String,
    components: Vec<Component>,
}

impl ValidatedResponse {
    fn from_parsed_response(parsed_response: ParsedResponse) -> Option<Self> {
        let mut components = Vec::new();
        let description = parsed_response.description;
        let html = parsed_response.main_html;

        for (description, html) in parsed_response
            .component_list
            .into_iter()
            .zip(parsed_response.component_html)
        {
            components.push(Component::new(description, html)?);
        }

        if components.is_empty() {
            return None;
        }

        Some(ValidatedResponse {
            description,
            html,
            components,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Component {
    name: String,
    is_standalone: bool,
    description: String,
    html: String,
}

impl Component {
    fn new(description: ComponentDescription, html: ComponentHtml) -> Option<Self> {
        // make sure the names match
        let name = description.name.trim();
        if name.to_lowercase() != html.name.trim().to_lowercase() {
            return None;
        }

        let is_standalone = description.is_standalone;

        let description = description.description.trim().to_string();
        let html = html.html.trim().to_string();

        // If it says it's a standalone component, make sure it doesn't contain {children}
        if is_standalone {
            if html.contains("{children}") {
                return None;
            }
        }
        // If it says it's not a standalone component, make sure it contains {children}
        else if !html.contains("{children}") {
            return None;
        }

        Some(Component {
            name: name.to_string(),
            is_standalone,
            description,
            html,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ParsedResponse {
    description: String,
    component_list: Vec<ComponentDescription>,
    main_html: String,
    component_html: Vec<ComponentHtml>,
}

impl ParsedResponse {
    fn new(content: &str) -> Option<Self> {
        // try to find the responses to each part of the prompt. They may be marked by numbers, or the question
        let split_strategy = SplitStrategy::detect(content)?;
        let responses = split_strategy.split(content)?;

        let [description, component_list, main_html, component_html] = responses;

        let component_list = component_list
            .lines()
            .filter(|line| {
                !line.chars().all(|c| c.is_whitespace()) && !line.trim_end().ends_with(':')
            })
            .map(ComponentDescription::parse)
            .collect::<Option<Vec<_>>>()?;

        let component_html = ComponentHtml::parse(&component_html)?;

        // Filter any lines between the ```html and ``` tags
        let main_html_lines = main_html.lines();
        let mut main_html = String::new();
        let mut in_html = false;
        for line in main_html_lines {
            if line.contains("```") {
                if in_html {
                    break;
                }
                in_html = true;
            } else if in_html {
                main_html.push_str(line);
                main_html.push('\n');
            }
        }

        if !in_html {
            return None;
        }

        Some(ParsedResponse {
            description,
            component_list,
            main_html,
            component_html,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
enum SplitStrategy {
    // Split the prompt at #)
    NumberParen,
    // Split the prompt at #.
    NumberDot,
    // Split the prompt at each question
    Question,
}

impl SplitStrategy {
    fn detect(prompt: &str) -> Option<Self> {
        let as_lower = prompt.to_lowercase();

        let has_all_questions = QUESTIONS.iter().all(|q| as_lower.contains(q));
        if has_all_questions {
            return Some(SplitStrategy::Question);
        }

        let paren_match_count: usize = (1..QUESTIONS.len())
            .map(|i| as_lower.matches(&format!("{})", i)).count())
            .sum();
        if paren_match_count == QUESTIONS.len() {
            return Some(SplitStrategy::NumberParen);
        }

        let dot_match_count: usize = (1..QUESTIONS.len())
            .map(|i| as_lower.matches(&format!("{}.", i)).count())
            .sum();
        if dot_match_count == QUESTIONS.len() {
            return Some(SplitStrategy::NumberDot);
        }

        None
    }

    fn split(&self, prompt: &str) -> Option<[String; 4]> {
        // We find matches for each line and then split the prompt at that whole line
        let mut current_response = String::new();
        let mut responses = Vec::new();

        let add_line = |current_response: &mut String, line: &str| {
            current_response.push('\n');
            *current_response += line;
        };

        let finish_response = |responses: &mut Vec<String>, current_response: &mut String| {
            let response = std::mem::take(current_response).trim().to_string();
            if !response.is_empty() {
                responses.push(response);
            }
        };

        match self {
            SplitStrategy::NumberParen => {
                let mut current_response_number = 1;
                let mut needle = format!("{})", current_response_number);
                for line in prompt.lines() {
                    if line.to_lowercase().contains(&needle) {
                        if current_response_number == 1 {
                            current_response.clear();
                        }
                        current_response_number += 1;
                        needle = format!("{})", current_response_number);
                        finish_response(&mut responses, &mut current_response);
                    } else {
                        add_line(&mut current_response, line);
                    }
                }
            }
            SplitStrategy::NumberDot => {
                let mut current_response_number = 1;
                let mut needle = format!("{}.", current_response_number);
                for line in prompt.lines() {
                    if line.to_lowercase().contains(&needle) {
                        if current_response_number == 1 {
                            current_response.clear();
                        }
                        current_response_number += 1;
                        needle = format!("{}.", current_response_number);
                        finish_response(&mut responses, &mut current_response);
                    } else {
                        add_line(&mut current_response, line);
                    }
                }
            }
            SplitStrategy::Question => {
                let mut before_question = true;
                let mut question_iter = QUESTIONS.iter().peekable();
                for line in prompt.lines() {
                    if let Some(question) = question_iter.peek() {
                        if line.to_lowercase().contains(**question) {
                            if before_question {
                                current_response.clear();
                                before_question = false;
                            }
                            finish_response(&mut responses, &mut current_response);
                            question_iter.next();
                        } else {
                            add_line(&mut current_response, line);
                        }
                    } else {
                        add_line(&mut current_response, line);
                    }
                }
            }
        };

        current_response.push('\n');
        responses.push(std::mem::take(&mut current_response));

        if responses.len() != QUESTIONS.len() {
            return None;
        }

        Some(responses.try_into().unwrap())
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ComponentDescription {
    name: String,
    is_standalone: bool,
    description: String,
}

impl ComponentDescription {
    fn new(name: String, is_standalone: bool, description: String) -> Self {
        Self {
            name,
            is_standalone,
            description,
        }
    }

    fn parse(line: &str) -> Option<Self> {
        const STANDALONE: &str = "stand";
        const CHILD: &str = "child";

        let lower = line.to_lowercase();
        let contains_stand = lower.contains(STANDALONE);
        let contains_child = lower.contains(CHILD);

        if (contains_stand && contains_child) || (!contains_stand && !contains_child) {
            return None;
        }

        let is_standalone = contains_stand;

        let mut description = String::new();

        // Split the line at the first word
        let mut words = line.split_whitespace();

        let name = take_component_name(&mut words)?;

        for word in words {
            description.push_str(word);
            description.push(' ');
        }

        Some(ComponentDescription::new(name, is_standalone, description))
    }
}

fn take_component_name(words: &mut std::str::SplitWhitespace) -> Option<String> {
    let mut name = None;
    while name.is_none() {
        let word = words.next()?;
        let filtered = word
            .trim()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        if filtered.is_empty() {
            continue;
        }
        let first_char = filtered.chars().next()?;
        let starts_with_upper = first_char.is_uppercase();
        let starts_with_number = first_char.is_numeric();
        let valid_name = starts_with_upper && !starts_with_number;
        if valid_name {
            name = Some(filtered);
        }
    }

    name
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct ComponentHtml {
    name: String,
    html: String,
}

impl ComponentHtml {
    fn parse(components_html: &str) -> Option<Vec<Self>> {
        let mut component_html = Vec::new();

        let mut current_component_html: Option<String> = None;
        let mut current_name = None;

        for line in components_html.lines() {
            if line.chars().all(|c| c.is_whitespace()) {
                continue;
            }

            if line.contains("```") {
                // If we are in html for a component, we need to finish it
                if let Some(current_html) = current_component_html.take() {
                    component_html.push(ComponentHtml {
                        name: current_name.take()?,
                        html: current_html,
                    })
                } else {
                    current_component_html = Some(String::new());
                    // If we started parsing html, but we aren't in a component, this is invalid
                    current_name.as_ref()?;
                }
            } else if let Some(current_component_html) = current_component_html.as_mut() {
                current_component_html.push_str(line);
            }
            // Check if this is the html for a component
            else {
                // This is a new component, find the first word which is the name
                let mut words = line.split_whitespace();
                let name = take_component_name(&mut words)?;

                current_name = Some(name);
            }
        }

        Some(component_html)
    }
}
