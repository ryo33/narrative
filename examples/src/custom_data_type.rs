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
    const USER_ID: UserId = UserId::new("user123");

    #[step("User {id:?} logs in as {role:?}", id = USER_ID, role = UserRole::Admin)]
    fn user_logs_in(id: UserId, role: UserRole);

    #[step("User has access to admin panel", has_access = true)]
    fn check_admin_access(has_access: bool);

    #[step("List all users", users = vec![UserRecord::new("user123", "Alice", UserRole::Admin), UserRecord::new("user456", "Bob", UserRole::User)])]
    fn list_all_users(users: Vec<UserRecord>);
}

// Custom data types with #[local_type_for] macro
// This implements both IndependentType and UserStoryLocalType
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
    #[allow(dead_code)]
    User,
    #[allow(dead_code)]
    Guest,
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(UserStory)]
pub struct UserRecord {
    id: UserId,
    name: &'static str,
    role: UserRole,
}

impl UserRecord {
    pub const fn new(id: &'static str, name: &'static str, role: UserRole) -> Self {
        Self { id: UserId::new(id), name, role }
    }
}

struct UserStoryEnv {
    current_user_id: Option<&'static str>,
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

    fn list_all_users(&mut self, users: Vec<UserRecord>) -> Result<(), Self::Error> {
        for user in users {
            println!("User: {} ({:?})", user.name, user.id);
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
