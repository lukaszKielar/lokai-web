# LokAI

LokAI is a self-hosted ChatGPT-like AI assistant, that integrates with [Ollama](https://ollama.com/).
The goal of this project was to learn something about [Leptos](https://leptos.dev/)), LLMs and Rust.
There are many flaws in it, and I couldn't call it a production grade application, but I had tonne of fun working on it, and I hope it may be an inspiration for some more ambitious projects out there.

<video src="https://github.com/lukaszKielar/lokai/assets/31779738/2abbab35-5add-45c9-a8e6-80de75b6549f"></video>

Before you run the app, make sure you have Ollama server installed and running:

```bash
ollama serve
# or with docker
docker run -v ~/.docker-share/ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama # runs once
docker stop ollama
docker start ollama
```

So far the name of the LLM model is hardcoded in the app, and if you don't want to wait ages for the response, you might want to download the model prior to the app, you can do so by typing:

```bash
ollama pull phi3:3.8b
```

To run the project locally with hot-reloading, type:

```bash
cargo leptos watch
```

Once it's done you can navigate to http://localhost:3000 and start conversating with it.

## Development

More to come...

## Licensing

Project is licensed under the MIT license.

## Acknowledgement

This project took an inspiration from [Monte9/nextjs-tailwindcss-chatgpt-clone](https://github.com/Monte9/nextjs-tailwindcss-chatgpt-clone) and [MoonKraken/rusty_llama](https://github.com/MoonKraken/rusty_llama).

## Watch

```bash
cargo watch -w src -w templates -w assets -x run
```
