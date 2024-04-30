use std::path::PathBuf;

use data_gen::{Chat, Train, PROMPT, UI_COMPONENTS};
use fundu::parse_duration;
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

#[tokio::main]
async fn main() {
    let client = Client::new();

    let mut responses = Vec::new();
    let mut data_count = 0;

    loop {
        let mut iter = UI_COMPONENTS.iter().peekable();
        while let Some(component) = iter.peek() {
            let component = component.trim();
            println!("Generating for {}", component);

            let groq_key = std::env::var("GROQ_KEY").unwrap();

            let Ok(res) = client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {groq_key}"))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "messages": [
                        {
                            "role": "system",
                            "content": PROMPT
                        },
                        {
                            "role": "user",
                            "content": component
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
                prompt: component.to_string(),
                response: chat,
            });

            // It was successful, so we can move on to the next component
            _ = iter.next();
            data_count += 1;

            if data_count % 10 == 0 {
                // write the data into a file
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let folder = PathBuf::from("data");
                let file = folder.join(format!("data{}.json", timestamp));
                let mut file = File::create(file).await.unwrap();
                let json = serde_json::to_string(&responses).unwrap();
                file.write_all(json.as_bytes()).await.unwrap();
            }
        }
    }
}
