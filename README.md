# Bedrust 🦀🧠⛅🪨

![screenshot of bedrust](/img/bedrust.png)

A neat way to invoke models on [Amazon Bedrock](https://aws.amazon.com/bedrock/). Written in Rust, and LIVE on [Twitch](https://twitch.tv/ruptwelve).

Currently supporting the following models:
- **Claude 3.5 Sonnet**
- Claude V2
- Claude V3 Sonnet
- Claude V3 Haiku
- Llama2 70B
- Cohere Command
- Jurrasic 2 Ultra
- Titan Text Express V1
- Mistral AI models (Mixtral, Mistral7b and Mistral Large 1 and 2)

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

### Make sure you have Rust and requrements installed 🦀

Well that just makes sense, this is a **Rust** application. The easiest way to get started is by using [rustup](https://www.rust-lang.org/tools/install). 

Now, you need some additional packages to be able to compile **bedrust**. Namely you need the `build-essential` (or similar) package group. Depending on your operating system, and package manager the name may differ.

**Ubuntu/Debian:**
```
sudo apt install build-essential
```

**Arch Linux:**
```
sudo pacman -S base-devel
```

**MacOS:**
```
xcode-select --install
```

**Amazon Linux/Red Hat/CentOS:**
```
yum groupinstall "Development Tools"
```

### Install it locally

To install the application locally, just run:
```
cargo install bedrust
```
This will install the compiled binary into your `$CARGO_HOME/bin` directory. If you have the `$PATH` set up correctly you should be able to run it now. But before you do ...

Let's initialize the configuration. Because **bedrust** uses a configuration file (`bedrust_config.ron`) it (along with some other resources) needs to be stored inside of your `$HOME/.config/bedrust` directory. *Now*, you can do this manually, but we have a feature to do it for you. Just run:
```
bedrust --init
```
You will get asked to pick a default model. And this will create all the necessary files for you to be able to use **bedrust**. There is no need to modify these files, unless you want to.

### Running the application 🚀

Finally, to run the application just use the following command:
```bash
bedrust -m <MODELNAME> # replacing the model name with one of the supported ones
```
Or if you wish to use the default model (the one defined during `--init` / in your config file) just run `bedrust` without any parameters. If you do not select a model by passing the `-m` parameter, AND you do not have a default model set in your config file, you will be prompted to pick one during the run.

## Usage
```bash
Usage: bedrust [OPTIONS]

Options:
      --init
  -m, --model-id <MODEL_ID>  [possible values: llama270b, cohere-command, claude-v2, claude-v21, claude-v3-sonnet, claude-v3-haiku, claude-v35-sonnet, jurrasic2-ultra, titan-text-express-v1, mixtral8x7b-instruct, mistral7b-instruct, mistral-large, mistral-large2]
  -c, --caption <CAPTION>
  -s, --source <SOURCE>
  -x
  -h, --help                 Print help
  -V, --version              Print version
```
Once, prompted enter your question, and hit `ENTER`. 🚀 To quit the program, just type `/q` in your question prompt.

## Captioning images

![screenshot of bedrust running the captioner](/img/captioner.png)

🚀 **NEW feature:** Thanks to the multimodality of Claude V3, you can now pass images to this Large Language Model. This means we can do some fun things like caption images for the sake of accessibility. This feature is available in Bedrust from version `0.5.0`.

> ⚠️ Currently the only two models that support this are: Claude V3 Sonnet, and Claude V3 Haiku

To use captioning you just need to pass it the `-c` parameter, along with the directory where you have your images:

```bash
bedrust -m claude-v3-sonnet -c /tmp/test-images/
```
This will retrieve the supported images, and produce captions for them. Ultimately producing a `captions.json` file in the current working directory with the captions connected to image paths.

Here is an example of the output:
```json
[
  {
    "path": "/tmp/test-images/4slika.jpeg",
    "caption": "A computer CPU fan cooling a circuit board with Ethernet and other ports."
  },
  {
    "path": "/tmp/test-images/kompjuter.jpeg",
    "caption": "An open circuit board with various electronic components and wires, placed in an office or workshop setting with shelves and equipment visible in the background."
  },
  {
    "path": "/tmp/test-images/c64.jpeg",
    "caption": "Vintage Commodore computer monitor displaying the Twitch logo on the screen."
  }
]
```

Additionally you can customize captioning *prompt* and *supported image file formats* by editing the `bedrust_config.ron` file in the root of this project.

## ⚠️  BETA FEATURE - Source Code analysis

You can now point Bedrust to a directory containing some source code. This will allow you to discuss your code repository in context, and it can provide you with code suggestions, improvements, and further development. 

> *Note:* Since this is a beta feature, it has it's limitations. For example, it is not able to handle really big code bases. And because it sends your entire code base into the context, it may cost you significantly more.

```bash
bedrust --source ~/workspace/repos/your_code_repo
```

## Configuration files 

There is one important configuration file that ship with **bedrust**:

- `bedrust_config.ron` - stores configuration parameters related to the application itself.

They *need* to be in your `$HOME/.config/bedrust/` directory. The application will warn you if they do not exist, and fail to run. You can create them automatically by running `bedrust --init`

## TODO
- [x] Ability to get user input
- [x] Being able to select a model
- [x] Have a conversation with the model
- [x] Stream the responses back word by word
- [x] Better error handling
- [ ] Code Testing
- [ ] Ability to generate images
- [x] Make it prettier
- [ ] Handle long pastes Better
- [x] Bedder credential handling
