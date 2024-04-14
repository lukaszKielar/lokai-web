use cfg_if::cfg_if;
use leptos::{logging, server, ServerFnError};
use serde::{Deserialize, Serialize};

// TODO: I need to save a context of the chat into DB
// that would help when user decided to come back to old conversation
// I won't be feeding model with previous prompts
// asynchronously save everything to DB (maybe in batch mode?? - future consideration)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenerateResponse {
    pub done: bool,
    pub context: Vec<i32>,
    pub response: String,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        // const MODEL: &str = "mistral:7b";
        // const MODEL: &str = "gemma:7b";
        // const MODEL: &str = "gemma:2b";
        const MODEL: &str = "tinyllama";

        fn default_model() -> String {
            MODEL.to_string()
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct GenerateParams {
            #[serde(default = "default_model")]
            model: String,
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            context: Option<Vec<i32>>,
            #[serde(default)]
            stream: bool
        }

        async fn generate(prompt: String, context: Option<Vec<i32>>) -> GenerateResponse {
            let client = reqwest::Client::new();
            let params = GenerateParams {
                model: MODEL.to_string(),
                prompt: prompt,
                context: context,
                stream: false
            };
            logging::log!("request params: {:?}", params);

            let response: GenerateResponse = client.post("http://host.docker.internal:11434/api/generate").json(&params).send().await.unwrap().json().await.unwrap();

            logging::log!("response: {:?}", response);

            response
        }
    }
}

// TODO: save every prompt, response and context to database, async thread
// TODO: this function should take id of the conversation, prompt and context (history of conversation)
#[server(ReplAI, "/api")]
pub async fn replai(
    prompt: String,
    context: Option<Vec<i32>>,
) -> Result<GenerateResponse, ServerFnError> {
    let response = generate(prompt, context).await;

    Ok(response)
}
