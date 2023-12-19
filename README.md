# Narrative

An immensely simple library for story-driven development

## !!!! NOT RELEASED YET !!!!

WIP to the first release. Published to crates.io for reserve the name.

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

- **Story**: a sequence of steps, which is written as a trait
- **Step**: a single action or assertion in a story
- **Story Trait**: a macro-generated trait that represents a story. it has a
  method for each step.
- **Story Context**: a struct that holds the all related information or data of
  a story.
- **Story Env**: a data structure that implements a story trait.

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
    #[step("I have {count} orages", count = 2)]
    fn have_two_oranges(count: u32);
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}
```

Wow, it's neat!

3. Implement the story in Rust.

```rust
pub struct MyFirstStoryImpl {
    apples: u8,
    oranges: u8,
};

impl MyFirstStory for MyFirstStoryImpl {
    type Error = ();

    fn as_a_user(&mut self) -> Result<(), Self::Error> {
        println!("Hi, I'm a user");
        Ok(())
    }

    fn have_one_apple(&mut self, count: u32) -> Result<(), Self::Error> {
        self.apples = count;
        Ok(())
    }

    fn have_two_oranges(&mut self, count: u32) -> Result<(), Self::Error> {
        self.oranges = count;
        Ok(())
    }

    fn should_have_three_fruits(&mut self, total: u32) -> Result<(), Self::Error> {
        assert_eq!(self.apples + self.oranges, total);
        Ok(())
    }
}
```

You may notice that the signature of the trait methods is a bit different from
the declaration, but it's fine.

4. Use the story in your code.

```rust
fn main() {
    let mut story = MyFirstStory { apples: 0, oranges: 0 };
    // You can run the story, and get the result.
    let story_result = story.run_all();
    // You can run the story step by step.
    for step in story.get_context().steps() {
        let step_result = step.run();
    }
}
```

### Subtle but Important Points

There are several points that you should know to use Narrative.

#### Async one is also defined automatically

Story doesn't have to use async keyword, and both sync and async version are
defined automatically.

```rust
#[async_trait]
impl AsyncMyFirstStory for MyFirstStoryImpl {
    type Error = ();
    async fn as_a_user(&mut self) -> Result<(), Self::Error> {
        println!("Hi, I'm a user");
        Ok(())
    }
    async fn have_one_apple(&mut self, count: u32) -> Result<(), Self::Error> {
        self.apples = count;
        Ok(())
    }
    async fn have_two_oranges(&mut self, count: u32) -> Result<(), Self::Error> {
        self.oranges = count;
        Ok(())
    }
    async fn should_have_three_fruits(&mut self, total: u32) -> Result<(), Self::Error> {
        assert_eq!(self.apples + self.oranges, total);
        Ok(())
    }
}
```

#### Arguments of the step methods cannot be data structures not defined in standard library

It makes your stories truely independent from any implementation.

#### But, you can use trait coupled to the exact story as arguments

Rust's type system gives us a power to write correct codes without loosing
productivity, and it's the same in writing stories (in Narrative). To achieve
the benefits without adding any dependency to the story, we can define new
struct or trait that strongly coupled to only the story, and use it as an
associated type of the story trait.

Don't worry the collision of the trait/struct names, it has a separate namespace
than other stories.

```rust
#[narrative::story("This is my first story")]
trait MyFirstStory {
    struct UserName(String);

    trait UserId {
        /// Generate a new user id with random uuid v4.
        fn new_v4() -> Self;
    }

    let user_id = Self::UserId::new_v4();

    #[step("I'm a user with id: {id}", id = user_id, name = UserName("Alice".to_string()))]
    fn as_a_user(id: Self::UserId, name: UserName);
}
```

It's really weird for who knows correct Rust syntax, but it's the better one
among alternative ideas to do the same thing, defining a new struct or trait in
the same place.

#### You can forget about the actual implementation of a story while writing a story

We think that stories should not regard their actual implementations, so noisy
details like `async`, `&self`, `&mut self`, and `-> Result<(), Self::Error>` are
not required in the story definition. This surprising behavior can be mitigated
by using "Implement missing members" feature of rust-analyzer.

## Design Decisions

These decisions highlight Narrative's unique aspects, especially in comparison
to [Gauge](https://gauge.org/), a well-known end-to-end testing framework.

### ~~Narrative is designed to implement stories exclusively in Rust, (though it can still be used for testing projects in other languages.)~~

~~Supporting other languages in Narrative would introduce a lot of complexity in
its design, implementation, and usage. Narrative leverages Rust's core
functionality and rust-analyzer to provide rich development experience. Rust
wouldn't the best language for writing end-to-end tests for everyone, but, We
believe that it still has advantages in this area, with a great compiler,
robust, yet straightforward type system, and libraries from the vibrant
community.~~

Users can dynamically get story context, so you can implement steps in other
programming languages, and call them in dynamic way from Rust code:

```rust
fn execute_story(context: impl narrative::StoryContext) {
    for step in context.steps() {
        send_to_external_process(step.text(), step.arguments().map(|arg| Argument {
                name: arg.name(),
                ty: arg.ty(),
                debug: arg.debug(),
                json: step.serialize(serde_json::value::Serializer).unwrap(),
        }));
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

### Narrative uses declaration of trait for stories

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
