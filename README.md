# Bedrust

![screenshot of bedrust](/img/bedrust.png)

A "practical" way to invoke models on Amazon Bedrock. Written in Rust, and LIVE on [Twitch](https://twitch.tv/ruptwelve).

Currently supporting the following models:
- Claude V2
- Llama2 70B
- Cohere Command
- Jurrasic 2 Ultra
- Titan Text Express V1
- Mistral AI models (Mixtral and Mistral)

## Getting Started

To get started using this you need to do a few things:

### Get AWS credentials configured locally ☁️

To be able to interact with [Amazon Bedrock]() you need to have a set of AWS Credentials on the machine **Bedrust** will run on. The easiest way to get this set up, is by configuring the [AWS CLI](https://aws.amazon.com/cli/). Make sure to install the AWS CLI, and run the `aws configure` [command](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html) to set your credentials.

To verify if you have your AWS credentials set correctly, you can run `aws sts get-caller-identity`:
```bash
darko@devbox [~/workspace/projects/bedrust]: aws sts get-caller-identity
{
    "UserId": "AIDAXXXXXXXXXXXXXXXXXX5",
    "Account": "123456789999999",
    "Arn": "arn:aws:iam::123456789999999:user/alan-ford"
}
```
Oh, yeah, make sure the user whose credentials you configure has permissions to `InvokeModel` on Amazon Bedrock.

### Make sure you have Rust installed 🦀

Well that just makes sense, this is a **Rust** application. The easiest way to get started is by using [rustup](https://www.rust-lang.org/tools/install)

### Clone the Repository 💾

As of this date, there is no way to *install* this tool to your machine in a traditional sense. Some elements are hardcoded paths (ie the `model_config.ron` file). And thus, the way you use **Bedrust** is by cloning this repository somewhere locally:
```
git clone https://github.com/darko-mesaros/bedrust && cd bedrust
```

### Running the application 🚀

Finally, to run the application just use the following `cargo` command:
```bash
cargo run -- -m <MODELNAME> # replacing the model name with one of the supported ones
```

## Usage
```bash
Usage: bedrust --model-id <MODEL_ID>

Options:
  -m, --model-id <MODEL_ID>  [possible values: llama270b, cohere-command, claude-v2, claude-v21, jurrasic2-ultra, titan-text-express-v1, mixtral8x7b-instruct, mistral7b-instruct]
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
