#[narrative::story("This should fail because it's not a trait")]
struct NotATrait {
    field: String,
}

fn main() {}
