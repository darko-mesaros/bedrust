# Bedrust 🦀🧠⛅🪨

![screenshot of bedrust](/img/bedrust.png)

A neat way to invoke models on [Amazon Bedrock](https://aws.amazon.com/bedrock/). Written in Rust, and LIVE on [Twitch](https://twitch.tv/ruptwelve).

> *NEW AS OF 0.8.6* - You can add `.bedrustrules` to your source code repo, for custom instructions when dealing with said code.
> *NEW AS OF 0.8.2* - BETA: You can now export your chat to HTML files. (It will only save them as `conversation.html` in the current directory) 

Currently supporting the following models:
- Claude 3.5 v2 Sonnet
- Claude 3.5 Haiku
- Claude 3.5 Sonnet
- Claude V2
- Claude V3 Sonnet
- Claude V3 Opus
- Claude V3 Haiku
- Llama2 70B
- LLama3.1 models
- Cohere Command
- Jurrasic 2 Ultra
- Titan Text Express V1
- Mistral AI models (Mixtral, Mistral7b and Mistral Large 1 and 2)
- Amazon Nova models

## Getting Started

To get started using this you need to do a few things:

### Get AWS credentials configured locally ☁️

To be able to interact with [Amazon Bedrock](https://aws.amazon.com/bedrock/) you need to have a set of AWS Credentials on the machine **Bedrust** will run on. The easiest way to get this set up, is by configuring the [AWS CLI](https://aws.amazon.com/cli/). Make sure to install the AWS CLI, and run the `aws configure` [command](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html) to set your credentials.

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
A command line tool to invoke and work with Large Language models on AWS, using Amazon Bedrock

Usage: bedrust [OPTIONS]

Options:
      --init
  -m, --model-id <MODEL_ID>  [possible values: llama270b, llama31405b-instruct, llama3170b-instruct, llama318b-instruct, cohere-command, claude-v2, claude-v21, claude-v3-opus, claude-v3-sonnet, claude-v3-haiku, claude-v35-sonnet, claude-v352-sonnet, claude-v35-haiku, jurrasic2-ultra, titan-text-express-v1, mixtral8x7b-instruct, mistral7b-instruct, mistral-large, mistral-large2, nova-micro, nova-lite, nova-pro]
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

## ⚠️  BETA FEATURE - Chat saving, recalling and export

![screenshot of the chat export feature](/img/chat_export.png)

As of version 0.8.2 you can now save your conversations, recall them at a later time, and even export them as nice HTML files. This feature is still in *heavy beta*, so expect things to break and functionality to change.

The way this works is, when you enter `/s` as a chat command, Bedrust saves your conversation inside of `~/.config/bedrust/chats` as a `.json` file. This fill will contain a generated summary and a title for the conversation. To recall the conversation you can just type `/r` as a chat command, and you will be able to select any of the saved ones.

To export your conversation to HTML, just run `/h`. This will create a file called `conversation.html` in the current directory. I have not yet implemented a feature to choose where to save this file, so for the time being it's just like this. (It's in beta afterall 😅).

## Configuration files 

There is one important configuration file that ship with **bedrust**:

- `bedrust_config.ron` - stores configuration parameters related to the application itself.

They *need* to be in your `$HOME/.config/bedrust/` directory. The application will warn you if they do not exist, and fail to run. You can create them automatically by running `bedrust --init`

## Instructions for code review

When passing the `--source` option, you can also pass some instruction to Bedrust. Ie, some rules, or a guide how to help you with your code. Think of it as an [system prompt](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/system-prompts) that Bedrust will use when responding to your questions about your code.

To do this, just place a file called `.bedrustrules` in *root of your code repository*. If this file does not exist, bedrust will continue to use the default system prompt as defined in `src/constants.rs`.

Here is an example of one:

```markdown
You are an expert in Rust, async programming, and concurrent systems.

Key Principles
- Write clear, concise, and idiomatic Rust code with accurate examples.
- Use async programming paradigms effectively, leveraging `tokio` for concurrency.
- Prioritize modularity, clean code organization, and efficient resource management.
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`).
- Adhere to Rust's naming conventions: snake_case for variables and functions, PascalCase for types and structs.
- Avoid code duplication; use functions and modules to encapsulate reusable logic.
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system.

Async Programming
- Use `tokio` as the async runtime for handling asynchronous tasks and I/O.
- Implement async functions using `async fn` syntax.
- Leverage `tokio::spawn` for task spawning and concurrency.
- Use `tokio::select!` for managing multiple async tasks and cancellations.
- Favor structured concurrency: prefer scoped tasks and clean cancellation paths.
- Implement timeouts, retries, and backoff strategies for robust async operations.

Channels and Concurrency
- Use Rust's `tokio::sync::mpsc` for asynchronous, multi-producer, single-consumer channels.
- Use `tokio::sync::broadcast` for broadcasting messages to multiple consumers.
- Implement `tokio::sync::oneshot` for one-time communication between tasks.
- Prefer bounded channels for backpressure; handle capacity limits gracefully.
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock` for shared state across tasks, avoiding deadlocks.

Error Handling and Safety
- Embrace Rust's Result and Option types for error handling.
- Use `?` operator to propagate errors in async functions.
- Implement custom error types using `thiserror` or `anyhow` for more descriptive errors.
- Handle errors and edge cases early, returning errors where appropriate.
- Use `.await` responsibly, ensuring safe points for context switching.

Testing
- Write unit tests with `tokio::test` for async tests.
- Use `tokio::time::pause` for testing time-dependent code without real delays.
- Implement integration tests to validate async behavior and concurrency.
- Use mocks and fakes for external dependencies in tests.

Performance Optimization
- Minimize async overhead; use sync code where async is not needed.
- Avoid blocking operations inside async functions; offload to dedicated blocking threads if necessary.
- Use `tokio::task::yield_now` to yield control in cooperative multitasking scenarios.
- Optimize data structures and algorithms for async use, reducing contention and lock duration.
- Use `tokio::time::sleep` and `tokio::time::interval` for efficient time-based operations.

Key Conventions
1. Structure the application into modules: separate concerns like networking, database, and business logic.
2. Use environment variables for configuration management (e.g., `dotenv` crate).
3. Ensure code is well-documented with inline comments and Rustdoc.

Async Ecosystem
- Use `tokio` for async runtime and task management.
- Leverage `hyper` or `reqwest` for async HTTP requests.
- Use `serde` for serialization/deserialization.
- Use `sqlx` or `tokio-postgres` for async database interactions.
- Utilize `tonic` for gRPC with async support.

Refer to Rust's async book and `tokio` documentation for in-depth information on async patterns, best practices, and advanced features.
```

You can find way more examples of this on [promptz.dev](https://www.promptz.dev/) and [cursor.directory](https://cursor.directory/)

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
