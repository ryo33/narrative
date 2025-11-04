# Narrative

An immensely simple library for story-driven development

## Overview

Narrative is a library dedicated to developing a whole or some part of software
based on stories expressed in a Rust trait. Though its primary design is for
end-to-end testing, its simplicity supports a variety of use cases.

## Goals

- **Story-driven**: Code respects story, not the other way around
- **Data-driven**: Enabling stories to include structured data
- **No additional tooling**: Eliminating the need for extra installation and
  learning
- **Leverage existing ecosystem**: Rich experience with less implementation
- **Zero runtime cost**: Stories are processed at compile time

## Terminology

Key terms in this library are:

- **Story**: a sequence of steps, written as a trait
- **Step**: a single action or assertion in a story
- **Story Trait**: a macro-generated trait that represents a story, with a
  method for each step
- **Story Context**: a struct that holds metadata about a story and provides
  methods to run it
- **Story Env**: a struct that implements a story trait

## Usage

1. Add [narrative](https://crates.io/crates/narrative) to your cargo
   dependencies.

2. Write your first story as a trait.

```rust
#[narrative::story("This is my first story")]
trait MyFirstStory {
    #[step("Hi, I'm a user")]
    fn as_a_user();
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    #[step("I have {count} oranges", count = 2)]
    fn have_two_oranges(count: u32);
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}
```

Wow, it's neat!

3. Implement the story in Rust.

```rust
struct MyFirstStoryEnv {
    sum: u32,
}

impl MyFirstStory for MyFirstStoryEnv {
    type Error = std::convert::Infallible;

    fn as_a_user(&mut self) -> Result<(), Self::Error> {
        println!("Hi, I'm a user");
        Ok(())
    }

    fn have_one_apple(&mut self, count: u32) -> Result<(), Self::Error> {
        self.sum += count;
        Ok(())
    }

    fn have_two_oranges(&mut self, count: u32) -> Result<(), Self::Error> {
        self.sum += count;
        Ok(())
    }

    fn should_have_three_fruits(&mut self, total: u32) -> Result<(), Self::Error> {
        assert_eq!(self.sum, total);
        Ok(())
    }
}
```

You may notice that the signature of the trait methods is a bit different from
the declaration, but it's fine.

4. Use the story in your code.

```rust
use narrative::story::RunStory;

#[test]
fn test() {
    let mut env = MyFirstStoryEnv { sum: 0 };
    MyFirstStoryContext.run_story(&mut env).unwrap();
}
```

The `#[narrative::story]` macro generates a `MyFirstStoryContext` struct that
implements `StoryContext`, which provides methods to run the story and introspect
its structure.

### Features

#### Async support

Both sync and async traits are defined automatically. Prefix the trait name with
`Async` to implement the async version.

```rust
impl AsyncMyFirstStory for MyFirstStoryEnv {
    type Error = std::convert::Infallible;

    async fn as_a_user(&mut self) -> Result<(), Self::Error> {
        // async implementation
        Ok(())
    }
    // ... other async methods
}

#[test]
fn test_async() {
    use narrative::story::RunStoryAsync;
    let mut env = MyFirstStoryEnv { sum: 0 };
    futures::executor::block_on(MyFirstStoryContext.run_story_async(&mut env)).unwrap();
}
```

#### Constants

Define constants in the story trait to use in step text and arguments.

```rust
#[narrative::story("User Story")]
trait UserStory {
    const NAME: &str = "Alice";
    const ID: &str = "user123";

    #[step("User: {NAME}")]
    fn greet_user();

    #[step("Login as {name}", name = NAME)]
    fn login(name: &str);

    #[step("Visit profile", url = format!("https://example.com/{ID}"))]
    fn visit_profile(url: String);
}
```

#### Custom data types

Use `#[narrative::local_type_for]` to define custom types for step arguments.
This keeps stories independent from implementation details while leveraging Rust's
type system.

```rust
#[narrative::story("User Management")]
trait UserStory {
    #[step("User {id:?} logs in as {role:?}", id = UserId::new("user123"), role = UserRole::Admin)]
    fn user_logs_in(id: UserId, role: UserRole);
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(UserStory)]
pub struct UserId(&'static str);

impl UserId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(UserStory)]
pub enum UserRole {
    Admin,
    User,
}
```

Types marked with `#[narrative::local_type_for]` can only be used in the specified
story, preventing coupling. Standard library types and common third-party types
(with `serde::Serialize`) like `uuid::Uuid`, `chrono::DateTime`, etc., can be used
directly without this attribute.

#### Sub stories

Stories can be composed by nesting them as steps. The parent step returns the
sub story implementation.

```rust
#[narrative::story("Setup")]
trait Setup {
    #[step("Initialize database")]
    fn init_db();
}

#[narrative::story("User Test")]
trait UserTest {
    #[step(story: Setup, "Run setup")]
    fn setup();

    #[step("Test user creation")]
    fn test_user();
}

impl UserTest for Env {
    type Error = std::convert::Infallible;

    fn setup(&mut self) -> Result<impl Setup<Error = Self::Error>, Self::Error> {
        Ok(SetupEnv { /* ... */ })
    }

    fn test_user(&mut self) -> Result<(), Self::Error> {
        // test implementation
        Ok(())
    }
}
```

#### Custom runners

Implement `StoryRunner` or `AsyncStoryRunner` to customize story execution,
add logging, reporting, or other cross-cutting concerns.

```rust
use narrative::runner::StoryRunner;

struct LoggingRunner;

impl<E> StoryRunner<E> for LoggingRunner {
    fn start_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        println!("Starting: {}", story.story_title());
        Ok(())
    }

    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: narrative::step::Step + narrative::step::Run<S, E>,
    {
        println!("Running: {}", step.step_text());
        step.run_with_runner(state, self)
    }

    // ... other methods
}

#[test]
fn test_with_runner() {
    let mut env = MyFirstStoryEnv { sum: 0 };
    let mut runner = LoggingRunner;
    MyFirstStoryContext.run_story_with_runner(&mut env, &mut runner).unwrap();
}
```

### Subtle but Important Points

#### Implementation details are omitted in story definitions

Stories should not be concerned with their implementation, so details like `async`,
`&mut self`, and `-> Result<(), Self::Error>` are not required in the trait
definition. The macro infers these from your implementation. Use rust-analyzer's
"Implement missing members" feature to generate the correct signatures.

## Design Decisions

These decisions highlight Narrative's unique aspects, especially in comparison
to [Gauge](https://gauge.org/), a well-known end-to-end testing framework.

### Narrative supports multi-language step implementations

Stories can be introspected at runtime, allowing step implementations in other
languages. The story context provides all metadata needed to dispatch steps to
external processes:

```rust
use narrative::story::StoryContext;

fn execute_story_externally(context: impl StoryContext) {
    for step in context.steps() {
        let args = step.args().map(|arg| {
            ExternalArg {
                name: arg.name(),
                ty: arg.ty(),
                debug: format!("{:?}", arg.value()),
                json: serde_json::to_value(arg.value()).unwrap(),
            }
        }).collect();
        send_to_external_process(step.step_text(), args);
    }
}
```

### Narrative is a library, not a framework

Narrative has no test runner, no plugin system, nor no dedicated language
server. Instead of being a framework, Narrative is a library that provides just
a single macro to implement stories. It's just a small tie between a story to a
plain Rust code. So, users can compose their own test runners or async runtime
with stories, and can use the full of rust-analyzer's functionality.

Narrative itself doesn't provide any features other than the core functionality,
declaring stories as traits and implementing them in Rust code. It lays the
groundwork for the simplicity and extensibility of this library.

The followings are the missing features in Narrative, and they never be
implemented in this library. But don't forget that you can do them by leveraging
the core features.

- step grouping
- story grouping
- test preparation and cleanup
- table-driven tests
- tags
- screenshoting
- retrying
- parallelization
- error reporting

### Narrative uses a declaration of trait to write a story

In other words, a story is an interface and step implementation depends on it.

Gauge uses markdown, and it's a great format for writing specifications,
documents, and stories while readable by non programmer. But, it's not the best
format for expressing data in structured way. We think story is more like a data
than a document, and it should be expressed in a structured way. With structured
data, we can leverage the power of software in the processing of them. In
Narrative, we use traits for expressing stories.

Using markdown for stories has another benefit, that is, it avoids the tight
coupling between stories and the implementation. If stories depends on specific
implementation, the story is not pure, and we loose many benefits of
story-driven development. One of the benefits is that we, including
non-programmer, can write stories freely without regard to the implementation,
and it gives us a kind of agility to the development.

But, it's not the case in Narrative though it let you write stories in Rust. In
Narrative, stories are written as traits, and it has no dependency to the
implementation, and it's just a contract between the story and the
implementation. Narrative would not loose the benefits of using markdown, on the
contray, it would make the situation better.

Narrative explicitly separates the story and the implementation, and it forces
the direction of the dependency. With markdown, we know that a story is the core
of the development, but occasionally we forget it or have a kind of cognitive
dissonance. It appeared to us as obvious experiences in the development, like,
"we need to know defined tags in the implementation to write a correct story",
"we have errors on the story editor if no step implementation", or "we failed to
write the correct story because the steps chosen from the editor's suggestion
are not implemented as we expect". In narrative, anyone can write stories
anytime, and stories written can exist as valid real properties with no error
even if implementation are completely undone.

The concept, a story is a contract to the implementation, makes the development
process and logical dependency graph clean and simple, and although it requires
a bit more effort to implement stories, it would give us a lot of benefits in
the long run of the development.

Someone might think that writing or reading Rust traits is impossible or
impractical to non-programmer, but we think it more optimistically. We are in
the era of many people can read and write code with the help of the great tools
and AIs, and, Personally, I believes clear codes wins documentation both for
programmers and non-programmers, and I don't think non-programmers cannot read
and write codes.

### Narrative encourages no reusing of steps

We encourage you to write your stories in fresh mind every time without reusing
existing steps, because we think stories should be self-contained. Being the
situation comes with big wins described below.

#### Accessibility for Newcomers

It empowers story writers that are not familiar with the existing codebase. They
don't need to know what steps already exist, to struggle with what steps to use,
and to worry about whether the chosen step is implemented as they expect.

#### Contextual Clarity

Copying steps from other stories often leads to a mix-up of contexts, and making
it not easy to decipher the key point of a story (without attaching proper
aliases to common steps). While we tend to have many story have the same steps
that shares the same context and implementation, it's challenging to maintain
the coherency of sharing the same logic while we add, remove, modify the
stories.

One downside of this approach is that stories could have inconsistency in the
writing style among them, but it can be mitigated by organizing stories in the
near each other with have the same contexts. It nudges writers to write stories
in a consistent way.

#### Simplicity

Reusing steps or group of steps could be a source of complexity. It's nightmare
to modify a step that is used by many stories without breaking them.

#### Fine-Grained Abstraction

A step is relatively large a unit for reuse or abstraction. Instead of sharing
the whole a step, we should share code between stories. But it should be done by
extracting common, story-agnostic, and atomic unit of logic. A step
implementation should be a composition of such units, and it should not leak the
story's context in the abstraction. For instance, if a step is about clicking a
submit button, it might be implemented as a composition of atomic logic like
`find_element_by(id)`, `click(element)`, and `wait_for_page_load()`, and not to
leak the context like `click_submit_button()` or `click_button("#submit")`.
