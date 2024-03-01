# Bedrust

![screenshot of bedrust](/img/bedrust.png)

A "practical" way to invoke models on Amazon Bedrock. Written in Rust, and LIVE on [Twitch](https://twitch.tv/ruptwelve).

Currently supporting the following models:
- Claude V2
- Llama2 70B
- Cohere Command
- Jurrasic 2 Ultra
- Titan Text Express V1

## Usage

```bash
Usage: bedrust --model-id <MODEL_ID>

Options:
  -m, --model-id <MODEL_ID>  [possible values: llama270b, cohere-command, claude-v2, jurrasic2-ultra, titan-text-express-v1]
  -h, --help                 Print help
  -V, --version              Print version
```

Once, prompted enter your question, and hit `ENTER`. 🚀

## TODO
- [x] Ability to get user input
- [x] Being able to select a model
- [ ] Have a conversation with the model
- [x] Stream the responses back word by word
- [ ] Better error handling
- [ ] Code Testing
- [ ] Ability to generate images
- [x] Make it prettier
