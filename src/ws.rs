use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    db,
    error::Result,
    ollama::{default_model, OllamaChatParams, OllamaChatResponseStream},
};
use crate::{models, state::AppState};

pub async fn websocket(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    debug!("start handling a socket");

    let (inference_request_tx, mut inference_request_rx) = mpsc::channel::<models::Message>(100);
    let (inference_response_tx, mut inference_response_rx) = mpsc::channel::<models::Message>(100);
    let (mut sender, mut receiver) = socket.split();

    let mut inference_thread = tokio::spawn(async move {
        info!("inference thread started");
        while let Some(user_prompt) = inference_request_rx.recv().await {
            let inference_response_tx_clone = inference_response_tx.clone();
            let state_clone = state.clone();
            match inference(user_prompt, inference_response_tx_clone, state_clone).await {
                Ok(_) => {}
                // consider sending a message through the channel to indicate failure
                Err(err) => {
                    error!(?err, "error while processing inference request, exiting...");
                    break;
                }
            };
        }
        info!("inference thread exited");
    });

    let mut sender_thread = tokio::spawn(async move {
        info!("ws sender thread started");
        while let Some(assistant_response_chunk) = inference_response_rx.recv().await {
            debug!(?assistant_response_chunk, "got assistant response chunk");
            let assistant_response_chunk_json =
                match serde_json::to_string(&assistant_response_chunk) {
                    Ok(value) => value,
                    Err(err) => {
                        error!(?err, "cannot serialise assistant response, exiting...");
                        break;
                    }
                };
            if sender
                .send(Message::Text(assistant_response_chunk_json))
                .await
                .is_err()
            {
                // client disconnected
                return;
            }
        }
        info!("ws sender thread exited");
    });

    let mut receiver_thread = tokio::spawn(async move {
        info!("ws receiver thread started");
        while let Some(Ok(Message::Text(user_prompt_form_msg))) = receiver.next().await {
            debug!(
                ?user_prompt_form_msg,
                "user prompt form data received through websocket"
            );
            let user_prompt: models::Message = match serde_json::from_str::<
                models::UserPromptFormMessage,
            >(&user_prompt_form_msg)
            {
                Ok(value) => {
                    debug!(?value, "deserialised data");
                    value.into()
                }
                Err(err) => {
                    error!(?err, "cannot deserialise user prompt, exiting...");
                    break;
                }
            };
            match inference_request_tx.send(user_prompt).await {
                Ok(_) => {}
                Err(err) => {
                    error!(?err, "cannot send inference request, exiting...");
                    break;
                }
            };
        }
        info!("ws receiver thread exited");
    });

    tokio::select! {
        inference_thread_result = (&mut inference_thread) => {
            match inference_thread_result {
                Ok(_) => info!("inference thread exited without errors"),
                Err(err) => error!(?err, "error returned by inference thread"),
            }
            warn!("aborting other threads");
            sender_thread.abort();
            receiver_thread.abort();
        },
        sender_thread_result = (&mut sender_thread) => {
            match sender_thread_result {
                Ok(_) => info!("sender thread exited without errors"),
                Err(err) => error!(?err, "error returned by sender thread"),
            }
            warn!("aborting other threads");
            inference_thread.abort();
            receiver_thread.abort();
        },
        receiver_thread_result = (&mut receiver_thread) => {
            match receiver_thread_result {
                Ok(_) => info!("receiver thread exited without errors"),
                Err(err) => error!(?err, "error returned by receiver thread"),
            }
            warn!("aborting other threads");
            sender_thread.abort();
            inference_thread.abort();
        },
    }

    debug!("finished handling a socket");
}

async fn inference(
    user_prompt: models::Message,
    inference_response_tx: mpsc::Sender<models::Message>,
    state: AppState,
) -> Result<()> {
    debug!(
        conversation_id = user_prompt.conversation_id.to_string(),
        "start inference"
    );
    let client = state.reqwest_client;

    // SAFETY: conversation exists at this point as we navigated from web browser and router validated this rule
    let conversation = db::get_conversation(state.sqlite.clone(), user_prompt.conversation_id)
        .await?
        .unwrap();
    let conversation_id = conversation.id;

    let mut messages = db::get_conversation_messages(state.sqlite.clone(), conversation_id).await?;
    messages.push(user_prompt.clone());

    {
        // TODO: send user_prompt right back
        let sqlite = state.sqlite.clone();
        let _ = db::create_message(sqlite, user_prompt).await?;
    }

    let params = OllamaChatParams {
        model: default_model(),
        messages: messages.into_iter().map(|m| m.into()).collect(),
        stream: true,
    };

    let mut stream = client
        .post("http://host.docker.internal:11434/api/chat")
        .json(&params)
        .send()
        .await?
        .bytes_stream()
        .map(|chunk| chunk.unwrap())
        .map(|chunk| serde_json::from_slice::<OllamaChatResponseStream>(&chunk));

    let mut assistant_response = models::Message::assistant("".to_string(), conversation_id);

    while let Some(chunk) = stream.next().await {
        if let Ok(chunk) = chunk {
            assistant_response.update_content(&chunk.message.content);

            let assistant_response_chunk =
                models::Message::assistant(chunk.message.content, conversation_id);
            if inference_response_tx
                .send(assistant_response_chunk)
                .await
                .is_err()
            {
                break;
            };

            if chunk.done {
                break;
            }
        }
    }

    let _ = db::create_message(state.sqlite, assistant_response).await;
    debug!(
        conversation_id = conversation_id.to_string(),
        "inference done"
    );

    Ok(())
}
