#[narrative::story("Using third party crates")]
trait ThirdPartyCrates {
    #[step("Generate a uuid", uuid = "14f95cf3-4302-4e59-9b49-e40cdc4c6ba3".parse().unwrap())]
    fn generate_uuid(&self, uuid: uuid::Uuid);

    #[step("Now is {now}", now = chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339("2025-04-28T00:00:00Z").unwrap())]
    fn jump_to_tomorrow(&self, now: chrono::DateTime<chrono::FixedOffset>);

    #[step("Print the json", json = serde_json::json!({
        "key": "value"
    }))]
    fn print_json(&self, json: serde_json::Value);
}

#[allow(dead_code)]
struct ThirdPartyCratesImpl {
    state: Vec<String>,
}

impl ThirdPartyCrates for ThirdPartyCratesImpl {
    type Error = std::convert::Infallible;

    fn generate_uuid(&mut self, uuid: uuid::Uuid) -> Result<(), Self::Error> {
        self.state.push(format!("Generated UUID: {uuid}"));
        Ok(())
    }

    fn jump_to_tomorrow(
        &mut self,
        now: chrono::DateTime<chrono::FixedOffset>,
    ) -> Result<(), Self::Error> {
        self.state.push(format!("Time jumped to: {now}"));
        Ok(())
    }

    fn print_json(&mut self, json: serde_json::Value) -> Result<(), Self::Error> {
        self.state.push(format!("JSON: {json}"));
        Ok(())
    }
}

#[test]
fn test_third_party_crates() {
    use narrative::story::RunStory as _;
    let mut env = ThirdPartyCratesImpl { state: vec![] };
    ThirdPartyCratesContext.run_story(&mut env).unwrap();
    assert_eq!(
        env.state,
        vec![
            "Generated UUID: 14f95cf3-4302-4e59-9b49-e40cdc4c6ba3",
            "Time jumped to: 2025-04-28 00:00:00 +00:00",
            "JSON: {\"key\":\"value\"}"
        ]
    );
}
