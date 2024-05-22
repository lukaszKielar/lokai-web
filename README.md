# LokAI

LokAI is a self-hosted ChatGPT-like AI assistant, that integrates with [Ollama](https://ollama.com/).

The goal of this project was to play around with [Ollama](https://ollama.com/) 🦀 **Rust** 🦀, [Axum](https://github.com/tokio-rs/axum), [Askama](https://github.com/djc/askama), [HTMX](https://htmx.org/) and [Hyperscript](https://hyperscript.org/).

I started project with [Leptos](https://leptos.dev/), but I spent too much time compiling, so I move towards something more lightweight.

Project has many flaws, but I had tonne of fun working on it, and I hope it may be an inspiration for some more ambitious projects out there.

<video src="https://github.com/lukaszKielar/lokai/assets/31779738/2abbab35-5add-45c9-a8e6-80de75b6549f"></video>

## Prepare env

### DevContainers

If you use VSCode and [DevContainers](https://containers.dev/) with the [plugin](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers), simply open project in IDE, and VSCode will recognise and build proper dev environment for you.

### Manual installs

To be able to develop and run the app locally, you need to install following:

-   Rust
-   cargo-watch
-   sqlx-cli
-   tailwindcss

You can sneak peak commands in [.devcontainer/Dockerfile](.devcontainer/Dockerfile).

## Running app

Before you run the app, make sure you have `Ollama` [installed](https://github.com/ollama/ollama).

When it's ready run:

```bash
ollama serve
# or with docker
docker run -v ~/.docker-share/ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama # runs once
docker stop ollama
docker start ollama
```

So far the name of the LLM model is hardcoded in the app (`phi3:3.8b`), and if you don't want to wait ages for the first response, you should download the model beforehand:

```bash
ollama pull phi3:3.8b
```

To run the project locally with hot-reloading, type:

```bash
cargo watch -x run
```

Once it's done, navigate to http://localhost:3000 and start playing around with it.

## Unit tests

Simply run:

```bash
cargo test
```

## Licensing

Project is licensed under the MIT license.

## Acknowledgements

This project took an inspiration from [Monte9/nextjs-tailwindcss-chatgpt-clone](https://github.com/Monte9/nextjs-tailwindcss-chatgpt-clone) and [MoonKraken/rusty_llama](https://github.com/MoonKraken/rusty_llama).
