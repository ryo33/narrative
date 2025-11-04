#![allow(dead_code)]
use std::convert::Infallible;

trait IndependentType {}
trait StoryLocalType {}

struct ExampleMyType;

impl<T: IndependentType> StoryLocalType for T {}
impl StoryLocalType for ExampleMyType {}

// Story trait that will generate UserStoryLocalType
#[narrative::story("User Story with Custom Data Types")]
trait UserStory {
    // TODO: Make this const once the consts-everywhere and lazy-consts feature are implemented.
    #[step("User {id:?} logs in as {role:?}", id = UserId::new("user123".to_string()), role = UserRole::Admin)]
    fn user_logs_in(id: UserId, role: UserRole);

    #[step("User has access to admin panel", has_access = true)]
    fn check_admin_access(has_access: bool);
}

// Custom data types with #[local_type_for] macro
// This implements both IndependentType and UserStoryLocalType
#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(UserStory)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(UserStory)]
pub enum UserRole {
    Admin,
    #[allow(dead_code)]
    User,
    #[allow(dead_code)]
    Guest,
}

struct UserStoryEnv {
    current_user_id: Option<String>,
    current_role: Option<UserRole>,
}

impl UserStory for UserStoryEnv {
    type Error = Infallible;

    fn user_logs_in(&mut self, id: UserId, role: UserRole) -> Result<(), Self::Error> {
        self.current_user_id = Some(id.0);
        self.current_role = Some(role);
        Ok(())
    }

    fn check_admin_access(&mut self, has_access: bool) -> Result<(), Self::Error> {
        if let Some(role) = &self.current_role {
            match role {
                UserRole::Admin => assert!(has_access),
                _ => assert!(!has_access),
            }
        }
        Ok(())
    }
}

#[test]
fn test() {
    use narrative::story::RunStory as _;
    let mut env = UserStoryEnv {
        current_user_id: None,
        current_role: None,
    };
    UserStoryContext.run_story(&mut env).unwrap();
}
