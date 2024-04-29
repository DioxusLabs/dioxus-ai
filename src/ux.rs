use kalosm::language::*;
use std::{fmt::Display, io::Write, ops::DerefMut};

use crate::pretty_print::get_clean_html;

pub async fn ux() {
    let llm = Llama::builder()
        .with_source(LlamaSource::phi_3_mini_4k_instruct())
        .build()
        .await
        .unwrap();

    let background =
        "The dioxus homepage is a marketing site for an open source UI library called dioxus.";

    let application_name = "dioxus homepage";

    let constraints = RegexParser::new(&format!(r#"\n1\. What did you expect to happen when you made the action\? [a-zA-Z:\.\-+'" ]{{1,1000}}\n2\. What changed in the new html\? [a-zA-Z:\.\-+'" ]{{1,1000}}\n3\. Does this behavior makes sense for {application_name}\? [a-zA-Z:\.\-+'" ]{{1,1000}}\n4\. Does this behavior make sense\? (yes|no)"#)).unwrap();

    let task = Task::builder(format!(r#"You are testing a web application that is currently in development called {application_name}. {background} You will receive the current HTML, an action and then the output HTML.
You must respond with this format:
1) What did you expect to happen when you made the action?
2) What changed in the new html?
3) Why does behavior makes sense or not for {application_name}?
4) Does this behavior make sense? (respond with yes, or no)"#))
        .with_constraints(constraints)
        .build();

    let mut should_be_yes = Vec::new();
    let mut should_be_no = Vec::new();

    {
        let buttons = ["component.rs", "async.rs", "server.rs", "global_state.rs"];
        // Navigate to dioxuslabs.com
        let tab = Tab::new("https://dioxuslabs.com".parse().unwrap(), false).unwrap();

        // Find the tabs div
        let node = tab.find("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div").unwrap();

        // Get the text
        let html = get_clean_html(&node).unwrap();

        for (i, button_name) in buttons.iter().enumerate() {
            println!("\n\nClicking element {} of {}", i + 1, buttons.len());

            // Click the button
            let button = tab.find(&format!("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div > div.flex-none.overflow-auto.whitespace-nowrap.flex.relative.min-w-full.bg-ghdarkmetal.pt-3.px-3 > ul > li:nth-child({}) > button", i + 2)).unwrap();
            button.click().unwrap();

            // Find the tabs div
            let node = tab.find("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div").unwrap();

            // Get the new text
            let new_html = get_clean_html(&node).unwrap();

            let prompt = Prompt {
                previous: html.clone(),
                action: format!("click the {button_name} button"),
                new: new_html,
            };

            should_be_yes.push(prompt);
        }
    }

    {
        let buttons = ["component.rs", "async.rs", "server.rs", "global_state.rs"];
        // Navigate to dioxuslabs.com
        let tab = Tab::new("https://dioxuslabs.com".parse().unwrap(), false).unwrap();

        // Find the tabs div
        let node = tab.find("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div").unwrap();

        // Get the text
        let html = get_clean_html(&node).unwrap();

        for (i, button_name) in buttons.iter().enumerate() {
            println!("\n\nClicking element {} of {}", i + 1, buttons.len());

            // Click the button
            let button = tab.find(&format!("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div > div.flex-none.overflow-auto.whitespace-nowrap.flex.relative.min-w-full.bg-ghdarkmetal.pt-3.px-3 > ul > li:nth-child({}) > button", i + 1)).unwrap();
            button.click().unwrap();

            // Find the tabs div
            let node = tab.find("#main > div > div:nth-child(6) > section:nth-child(1) > section:nth-child(2) > div.container.mx-auto.max-w-screen-lg > div > section > div").unwrap();

            // Get the new text
            let new_html = get_clean_html(&node).unwrap();

            let prompt = Prompt {
                previous: html.clone(),
                action: format!("click the {button_name} button"),
                new: new_html,
            };

            should_be_no.push(prompt);
        }
    }

    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;
    let mut true_negatives = 0;

    for prompt in should_be_yes {
        println!("\nPROMPT (should be yes):\n{prompt}\n");

        let mut all_text = String::new();
        let mut stream = task.run(&prompt.to_string(), &llm);
        while let Some(text) = stream.next().await {
            all_text.push_str(&text);
            print!("{text}");
            std::io::stdout().flush().unwrap();
        }

        if all_text.contains("Does this behavior make sense? yes") {
            true_positives += 1;
        } else if all_text.contains("Does this behavior make sense? no") {
            false_negatives += 1;
        }
    }

    for prompt in should_be_no {
        println!("\nPROMPT (should be no):\n{prompt}\n");

        let mut all_text = String::new();
        let mut stream = task.run(&prompt.to_string(), &llm);
        while let Some(text) = stream.next().await {
            all_text.push_str(&text);
            print!("{text}");
            std::io::stdout().flush().unwrap();
        }

        if all_text.contains("Does this behavior make sense? no") {
            true_negatives += 1;
        } else if all_text.contains("Does this behavior make sense? yes") {
            false_positives += 1;
        }
    }

    println!("\n\nTrue Positives: {}", true_positives);
    println!("False Positives: {}", false_positives);
    println!("False Negatives: {}", false_negatives);
    println!("True Negatives: {}", true_negatives);
}

struct Prompt {
    previous: String,
    action: String,
    new: String,
}

impl Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "You currently see:\n\n```html\n{}\n```",
            self.previous.trim()
        )?;

        write!(
            f,
            "\n\nYou {} and now see:\n\n```html\n{}\n```",
            self.action,
            self.new.trim()
        )?;

        Ok(())
    }
}
