use std::collections::HashMap;
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

            Some((x.prompt, ParsedResponse::new(content)?))
        })
        .collect::<Vec<(String, ParsedResponse)>>();
    println!("filtered: {}", filtered.len());

    let mut file = std::fs::File::create("filtered.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &filtered).unwrap();

    let validated = filtered
        .into_iter()
        .filter_map(|(prompt, parsed_response)| {
            Some((
                prompt,
                ValidatedResponse::from_parsed_response(parsed_response)?,
            ))
        })
        .collect::<Vec<(String, ValidatedResponse)>>();
    println!("validated: {}", validated.len());

    let mut combined = HashMap::new();
    for (prompt, validated_response) in validated {
        combined
            .entry(prompt)
            .or_insert(Vec::new())
            .push(validated_response);
    }

    println!("prompts deduplicated: {}", combined.len());

    let mut file = std::fs::File::create("validated.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &combined).unwrap();

    // Print a the first prompt and the first response
    let first_prompt = combined.keys().next().unwrap();
    println!("First prompt: {}", first_prompt);
    let first_response = combined.get(first_prompt).unwrap().first().unwrap();
    println!("First response: {}", first_response.description);
    println!("\nHTML:\n{}", first_response.html);
    println!("\nComponents:");
    for component in &first_response.components {
        println!("- {}", component.name);
        println!("{}", component.html);
    }

    // Write the finished prompts to a file
    let mut file = std::fs::File::create("finished_prompts.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &combined.keys().collect::<Vec<_>>()).unwrap();

    // Write the deduplicated data to a file
    let mut csv_writer = csv::Writer::from_path("data.csv").unwrap();
    for (prompt, mut validated_responses) in combined.clone() {
        let validated_response = validated_responses.pop().unwrap();
        let training_example = TrainingExample::new(prompt.to_string(), validated_response);
        csv_writer.serialize(training_example).unwrap();
    }

    // Write the full duplicated data to a file
    let mut csv_writer = csv::Writer::from_path("duplicated.csv").unwrap();
    for (prompt, validated_responses) in combined {
        for validated_response in validated_responses {
            let training_example = TrainingExample::new(prompt.to_string(), validated_response);
            csv_writer.serialize(training_example).unwrap();
        }
    }
}

// A normalized, validated training example
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct TrainingExample {
    pub input: String,
    pub output: String,
}

impl TrainingExample {
    fn new(prompt: String, validated_response: ValidatedResponse) -> Self {
        let description = &validated_response.description;
        let html = &validated_response.html;
        let component_descriptions = validated_response
            .components
            .iter()
            .map(|component| format!("- {}: {}", component.name, component.description))
            .collect::<Vec<String>>()
            .join("\n");
        let component_html = validated_response
            .components
            .iter()
            .map(|component| format!("{}:\n{}", component.name, component.html))
            .collect::<Vec<String>>()
            .join("\n");
        let response = format!(
            "DESCRIPTION:
{description}
COMPONENTS:
{component_descriptions}
HTML:
{html}
COMPONENT HTML:
{component_html}"
        );

        Self {
            input: prompt,
            output: response,
        }
    }
}

const QUESTIONS: &[&str] = &[
    "what should the ui look like?",
    "what are the individual components that make up the ui?",
    "what does the html for the ui look like?",
    "what is the html for each component?",
];

fn normalize_html(html: &str) -> String {
    let mut output = String::new();
    let mut after_node_before_non_whitespace = true;
    for c in html.chars() {
        let is_whitespace = c.is_whitespace();
        if after_node_before_non_whitespace && is_whitespace {
            continue;
        } else if c == '>' {
            output.push('>');
            after_node_before_non_whitespace = true;
        } else {
            output.push(c);
            after_node_before_non_whitespace = is_whitespace;
        }

        if c == '>' || c == '/' {
            output.pop();
            if output.ends_with(' ') {
                output.pop();
            }
            output.push(c);
        }
    }

    output.trim().replace("className", "class")
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct ValidatedResponse {
    description: String,
    html: String,
    components: Vec<Component>,
}

impl ValidatedResponse {
    fn from_parsed_response(parsed_response: ParsedResponse) -> Option<Self> {
        let mut components = Vec::new();
        let description = parsed_response.description.trim().to_string();
        let html = normalize_html(&parsed_response.main_html);

        for (description, html) in parsed_response
            .component_list
            .into_iter()
            .zip(parsed_response.component_html)
        {
            components.push(Component::new(description, html)?);
        }

        let mut myself = ValidatedResponse {
            description,
            html,
            components,
        };

        if myself.remove_unused_components() {
            return None;
        }

        if myself.contains_hallucinated_components() {
            return None;
        }

        if myself.components.is_empty() {
            return None;
        }

        Some(myself)
    }

    // Check if there are any hallucinated components
    fn contains_hallucinated_components(&mut self) -> bool {
        for html in self.html_iterator() {
            let without_whitespace = html
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>();

            let mut last_char_was_open = false;
            let mut building_component_name = false;
            let mut component_name = String::new();
            for c in without_whitespace.chars() {
                if c.is_whitespace() {
                    continue;
                }

                if c.is_ascii_uppercase() && last_char_was_open {
                    building_component_name = true;
                }

                if c == '/' || c == '>' {
                    if building_component_name {
                        let component_name_trimmed = component_name.trim();
                        if component_name_trimmed.is_empty() {
                            return true;
                        }

                        let component_exists = self
                            .components
                            .iter()
                            .any(|x| x.name.trim() == component_name_trimmed);
                        if !component_exists {
                            return true;
                        }

                        component_name.clear();
                        building_component_name = false;
                    }
                } else if building_component_name {
                    component_name.push(c);
                }

                last_char_was_open = c == '<';
            }
        }

        false
    }

    // Returns true if the response is invalid
    fn remove_unused_components(&mut self) -> bool {
        let mut used_components = Vec::new();
        for (i, component) in self.components.iter().enumerate() {
            let start_jsx = format!("<{}>", component.name);
            let end_jsx = format!("</{}>", component.name);
            let self_closing_jsx = format!("<{}/>", component.name);
            let mut used = false;
            for html in self.html_iterator() {
                let without_whitespace = html
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>();
                let contains_component = if component.is_standalone {
                    // If this is standalone, only allow self closing tags
                    if without_whitespace.contains(&start_jsx)
                        || without_whitespace.contains(&end_jsx)
                    {
                        return true;
                    }
                    without_whitespace.contains(&self_closing_jsx)
                } else {
                    // If this is not standalone, allow only a matching number of start and end tags
                    let contains_start = without_whitespace.matches(&start_jsx).count();
                    let contains_end = without_whitespace.matches(&end_jsx).count();
                    if contains_start != contains_end {
                        return true;
                    }
                    contains_start > 0
                };
                if contains_component {
                    used = true;
                    break;
                }
            }
            if used {
                used_components.push(i);
            }
        }

        if used_components.len() < self.components.len() {
            let mut use_components_iter = used_components.iter().peekable();
            for i in (0..used_components.len()).rev() {
                if let Some(next_used_component_index) = use_components_iter.peek().copied() {
                    if i == *next_used_component_index {
                        use_components_iter.next();
                    } else {
                        self.components.remove(i);
                    }
                }
            }
        }

        false
    }

    fn html_iterator(&self) -> impl Iterator<Item = &str> {
        self.components
            .iter()
            .map(|component| &*component.html)
            .chain(std::iter::once(&*self.html))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
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
        let html = normalize_html(&html.html);

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
            description: description.trim().to_string(),
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
