use component_structure::{Chat, Train};
use fundu::parse_duration;
use rand::prelude::SliceRandom;
use reqwest::Client;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};

const CATEGORY: &[&str] = &[
    "Social media",
    "Blog",
    "Product",
    "E-commerce",
    "News",
    "Jobs",
    "Events",
    "Health",
    "Finance",
    "Education",
    "Travel",
    "Entertainment",
    "Gaming",
    "Food",
    "Fitness",
    "Home",
    "Pets",
    "Personal",
    "Shopping",
    "Sports",
    "Technology",
    "Travel",
];

#[tokio::main]
async fn main() {
    let contents = std::fs::read_to_string("prompts_data.json").unwrap();
    let mut responses = serde_json::from_str::<Vec<Train>>(&contents).unwrap();
    // Find each line that start with `-` or `*` and add that to a list
    let mut prompts = Vec::new();
    for x in &responses {
        let content = &x.response.choices[0].message.content;
        for line in content.lines() {
            if let Some(prompt) = line.strip_prefix("- ").or(line.strip_prefix("* ")) {
                prompts.push(prompt.to_string());
            }
        }
    }

    // Write the prompts to a file
    std::fs::write("./prompts.json", serde_json::to_string(&prompts).unwrap()).unwrap();

    let client = Client::new();

    let mut data_count = 0;

    loop {
        let key = std::env::var("GROQ_KEY").unwrap();
        let category = CATEGORY.choose(&mut rand::thread_rng()).unwrap();
        let prompt = format!(
            r#"Imagine you are trying to build a website in the category of "{category}". Give me a description of the website first, then describe the UI components that are visual on the screen you would need to build that website. Respond with a flat list of bullet points for me to build those components.
                
Keep everything as precise as possible, don't include abstract/general concepts. Don't include anything that is interactive in the UI, just describe the visual elements. Eg. don't include a map, date picker, or a video game.

Respond with a series of requests like this:
- Create a button with a white outline
- Make a comment component that stands out from the black background
- A simple round avatar icon
- Make a component that displays a grid of children
- Header that includes a search bar and links to the documentation, support, and contact page
- Make something that shows a list of user testimonials"#,
        );

        let Ok(res) = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {key}"))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "model": "llama3-70b-8192",
                "seed": rand::random::<u64>()
            }))
            .send()
            .await
        else {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            continue;
        };

        let remaining_requests: u64 = res
            .headers()
            .get("x-ratelimit-remaining-requests")
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        let remaining_tokens: u64 = res
            .headers()
            .get("x-ratelimit-remaining-tokens")
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse().ok())
            .unwrap_or(0);
        let reset_requests = res
            .headers()
            .get("x-ratelimit-reset-requests")
            .and_then(|x| x.to_str().ok());
        let reset_requests = reset_requests
            .and_then(|reset_requests| parse_duration(reset_requests).ok())
            .unwrap_or(std::time::Duration::from_secs(60));
        let reset_tokens = res
            .headers()
            .get("x-ratelimit-reset-tokens")
            .and_then(|x| x.to_str().ok());
        let reset_tokens = reset_tokens
            .and_then(|reset_tokens| parse_duration(reset_tokens).ok())
            .unwrap_or(std::time::Duration::from_secs(60));
        println!("Remaining requests: {}", remaining_requests);
        println!("Remaining tokens: {}", remaining_tokens);
        println!("Reset requests: {:?}", reset_requests);
        println!("Reset tokens: {:?}", reset_tokens);

        if remaining_requests == 0 {
            println!("Rate limit reached, sleeping until reset");
            tokio::time::sleep(reset_requests).await;
            continue;
        }
        if remaining_tokens == 0 {
            println!("Rate limit reached, sleeping until reset");
            tokio::time::sleep(reset_tokens).await;
            continue;
        }

        let Ok(chat) = res.json::<Chat>().await else {
            tokio::time::sleep(reset_requests.max(reset_tokens)).await;
            continue;
        };
        let res = &chat.choices[0].message.content;
        println!("{}", res);

        responses.push(Train {
            prompt: Default::default(),
            response: chat,
        });

        data_count += 1;

        if data_count % 10 == 0 {
            // write the data into a file
            let folder = PathBuf::from("data");
            let file = folder.join("prompts.json");
            let mut file = File::create(file).await.unwrap();
            let json = serde_json::to_string(&responses).unwrap();
            file.write_all(json.as_bytes()).await.unwrap();
        }
    }
}
