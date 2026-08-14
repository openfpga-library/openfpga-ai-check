use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_transformers::{generation::LogitsProcessor, models::quantized_llama::ModelWeights};
use hf_hub::HFClient;
use tokenizers::Tokenizer;

use crate::{
    checks::{Check, CheckResult, CheckScore::Add},
    repo::RepoContext,
};

pub struct ReadmeCheck;

impl ReadmeCheck {}

#[async_trait]
impl Check for ReadmeCheck {
    fn name(&self) -> &'static str {
        "ReadmeCheck"
    }

    async fn run(&self, ctx: &RepoContext) -> anyhow::Result<Vec<CheckResult>> {
        let readme_content = ctx.readme_text().await?;
        let candidate_snippets = extract_candidate_snippets(&readme_content);
        let mut results: Vec<CheckResult> = vec![];

        for candidate_snippet in candidate_snippets {
            let ai_used = analyze_with_slm(&candidate_snippet).await?;
            if ai_used {
                results.push(CheckResult {
                    name: "README mentions AI usage".to_string(),
                    score: Add(0.5),
                    output: vec![candidate_snippet],
                });
            }
        }

        return Ok(results);
    }
}

fn extract_candidate_snippets(text: &str) -> Vec<String> {
    let keywords = [
        " ai ",
        " llm ",
        "gpt",
        "claude",
        "chatgpt",
        "deepmind",
        "openai",
        "anthropic",
        "llama",
        "mistral",
        "gemini",
        "ollama",
        "langchain",
        " rag ",
        "copilot",
        "large language model",
    ];

    let mut matched_sentences = Vec::new();
    for sentence in text.split(|c| c == '.' || c == '\n') {
        let lower_sentence = sentence.to_lowercase();
        if keywords.iter().any(|&k| lower_sentence.contains(k)) {
            let trimmed = sentence.trim();
            if !trimmed.is_empty() {
                matched_sentences.push(trimmed.to_string());
            }
        }
    }

    matched_sentences
}

async fn analyze_with_slm(snippet: &str) -> anyhow::Result<bool> {
    let client = HFClient::new()?;

    let model_repo = client.model("bartowski", "SmolLM2-360M-Instruct-GGUF");
    let model_path = model_repo
        .download_file()
        .filename("SmolLM2-360M-Instruct-Q5_K_M.gguf")
        .send()
        .await?;

    let tokenizer_repo = client.model("HuggingFaceTB", "SmolLM2-360M-Instruct");
    let tokenizer_path = tokenizer_repo
        .download_file()
        .filename("tokenizer.json")
        .send()
        .await?;

    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)?;

    println!("Loading weights into memory...");
    let mut file = std::fs::File::open(&model_path)?;
    let gguf_content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
    let mut model = ModelWeights::from_gguf(gguf_content, &mut file, &Device::Cpu)?;

    let prompt = format!(
        "<|im_start|>system\n\
        You are a binary classification assistant. Answer strictly with 'YES' or 'NO', followed by your reasoning.\n\
        Does the following README snippet explicitly state that the author used AI (e.g. Claude, ChatGPT, Copilot) to help write, generate, or assist with this code? \n\
        (Note: Merely mentioning AI concepts, hating on AI, or using an AI library does NOT count. The author must admit AI helped them write this project).\n\
        <|im_end|>\n\
        <|im_start|>user\n\
        README Snippet:\n\"{}\"\n\nAnswer YES or NO:\n\
        <|im_end|>\n\
        <|im_start|>assistant\n",
        snippet
    );

    let tokens = tokenizer.encode(prompt, true).map_err(anyhow::Error::msg)?;
    let mut tokens = tokens.get_ids().to_vec();

    println!("Generating response...");
    let mut logits_processor = LogitsProcessor::new(42, Some(0.1), None);
    let max_tokens = 150;
    let mut output_tokens = Vec::new();

    for index in 0..max_tokens {
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let start_pos = tokens.len().saturating_sub(context_size);

        let input = Tensor::new(&tokens[start_pos..], &Device::Cpu)?.unsqueeze(0)?;
        let logits = model.forward(&input, start_pos)?;
        let logits = logits.squeeze(0)?;

        let next_token = logits_processor.sample(&logits)?;
        tokens.push(next_token);

        if let Ok(text) = tokenizer.decode(&[next_token], false) {
            if text.contains("<|im_end|>") {
                break;
            }
            output_tokens.push(text);
        }
    }

    let output_text = output_tokens.join("").trim().to_string();
    println!("SLM Output: {}", output_text);

    let upper_output = output_text.to_uppercase();
    let is_ai_generated = upper_output.starts_with("YES") || upper_output.contains("YES");

    Ok(is_ai_generated)
}
