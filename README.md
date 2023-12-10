# Narrative

A simple library for logic implementation based on narrative stories.

## Status

WIP to the first release.

## Overview

Narrative is a library dedicated to developing a logic from stories expressed in
TOML. Though its primary design is for end-to-end testing, its simplicity
supports a variety of use cases.

## Goals

- **Story-driven**: Code respects story, not the other way around
- **Data-driven**: Enabling stories to include structured data
- **No additional tooling**: Eliminating the need for extra installation and
  learning
- **Leverage existing ecosystem**: Rich experience with less implementation
- **Zero runtime cost**: Stories are processed at compile time

## Terminology

Key terms in this library are:

- **Story**: a sequence of steps, which is written in a toml file
- **Step**: a single action or assertion in a story
- **Story Trait**: a macro-generated trait that represents a story. it has
  methods for each step.
- **Step ID**: a string that identifies a step. it allows us to rewrite a step
  text without breaking existing code.
- **Story Data**: a structured data that is associated with a story.
- **Step Data**: a structured data that is associated with a step.

## Usage

1. Add [narrative](https://crates.io/crates/narrative) to your cargo
   dependencies.

2. Write your first story in a TOML file.

```toml
[story]
title = "My first story"

[[step]]
id = "one apple"
text = "I have an apple"
data = { count = 1 }
[[step]]
id = "two oranges"
text = "I have {count} oranges"
data = { count = 2 }
[[step]]
id = "assert three fruits"
text = "I should have {count} fruits"
data = { count = 3 }
```

3. Implement the story in Rust.

```rust
use narrative::prelude::*;

story!("your-toml-file.toml");

pub struct MyFirstStory {
    apples: u8,
    oranges: u8,
};

impl NarrativeMyFirstStory for MyFirstStory {
    type Error = ();

    fn step_one_apple(&mut self, count: u32) -> Result<(), Self::Error> {
        self.apples = count;
    }

    fn step_two_oranges(&mut self, count: u32) -> Result<(), Self::Error> {
        self.oranges = count;
    }

    fn step_assert_three_fruits(&mut self) -> Result<(), Self::Error> {
        assert_eq!(self.apples + self.oranges, 3);
    }
}
```

4. Use the story in your code.

```rust
fn main() {
    let mut story = MyFirstStory { apples: 0, oranges: 0 };
    // You can get the story's title and steps.
    let narration = story.narrate();
    // You can run the story, and get the result.
    let story_result = story.run_all();
    // You can run the story step by step.
    for step in story.steps() {
        let step_result = step.run();
    }
}
```

## Design Decisions

These decisions highlight Narrative's unique aspects, especially in comparison
to [Gauge](https://gauge.org/), a well-known end-to-end testing framework.
framework.

### Narrative is designed to implement stories exclusively in Rust, (though it can still be used for testing projects in other languages.)

Supporting other languages in Narrative would introduce a lot of complexity in
its design, implementation, and usage. Narrative leverages Rust's core
functionality and rust-analyzer to provide an enough development experience.
Rust wouldn't the best language for writing end-to-end tests for everyone, but,
I believe that it still has advantages in this area, with a robust, yet
straightforward type system, and vibrant community driven libraries.

### Narrative is a library, not a framework

Narrative has no test runner, no plugin system, nor no rich IDE integration.
Instead of being a framework, Narrative is a library that provides just a single
macro (with a few variants) to implement stories. It's just a small tie between
a story to a plain Rust code. So, users can compose their own test runners,
async runtime, or IDE integration with stories.

### Narrative is feature-less

Narrative itself doesn't provide any features other than the core functionality,
writing stories in TOML files and implementing them in Rust. It lays the
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

### Narrative uses TOML for stories

Gauge uses markdown, and it's a great format for writing specifications,
documents, and stories mainly written by person than machine. But, it's not the
best format for expressing structured data. In Narrative, we use TOML for
stories, and it allows us to write stories in a more structured way.

### Narrative discourages reusing steps

We encourage you to write your stories in fresh mind every time without reusing
existing steps, because we think stories should be self-contained. Copy and
pasting steps from other stories would cause mixing of contexts, and it leads a
story to be hard to decipher the key point.

Also, reusing steps or group of steps could be a source of complexity. If a step
is dependent on many stories, it's hard to refine the step without breaking
other stories. So, self-contained stories are preferred in this library.

At implementation level, we encourage you to share code between stories. But it
should be done by extracting story-agnostic atomic logic. A step implementation
should be a composition of atomic logic, and it should not leak the story's
context in the abstraction of the atomic logic. For example, if a step is about
clicking a submit button, it should be implemented as a composition of atomic
logic like `find_element_by(id)`, `click(element)`, and `wait_for_page_load()`.
