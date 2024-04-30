use std::path::PathBuf;

use kalosm::language::*;

pub async fn generate() {
    let model = FileSource::local(PathBuf::from("./llama3-unsloth.Q4_K_M.gguf"));
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
        let input = prompt_input("What do you want to make? ").unwrap();
        let prompt = "<s>".to_string() + &input + "\nDESCRIPTION:\n";
        let start_timestamp = std::time::Instant::now();
        llm.stream_text(&prompt)
            .with_max_length(2048)
            .await
            .unwrap()
            .to_std_out()
            .await
            .unwrap();
        println!("\nTook: {:?}", start_timestamp.elapsed());
    }
}
