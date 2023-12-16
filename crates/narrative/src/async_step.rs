pub trait Step {
    type Story;
    type Error: std::error::Error;
    type StepId: StepId;
    fn run(&self, story: &mut Self::Story) -> Result<(), Self::Error>;
    fn step_text(&self) -> String;
    fn args(&self) -> StepArgs;
    fn id(&self) -> Self::StepId;
}

pub trait StepId: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + 'static {
    fn index(&self) -> usize;
}

pub trait Steps: IntoIterator {
    type Step: Step;
    fn get(&self, id: <Self::Step as Step>::StepId) -> &'static Self::Step;
    fn iter(&self) -> std::slice::Iter<'static, Self::Step>;
}

// This is concrete type because there nothing to be generic or hidden.
#[derive(Clone, Copy)]
pub struct StepArgs {
    args: &'static [StepArg],
}

impl StepArgs {
    pub fn new(args: &'static [StepArg]) -> Self {
        Self { args }
    }
    pub fn iter(&self) -> std::slice::Iter<'static, StepArg> {
        self.args.iter()
    }
}

impl IntoIterator for StepArgs {
    type Item = &'static StepArg;
    type IntoIter = std::slice::Iter<'static, StepArg>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

// This is concrete type because there nothing to be generic or hidden.
pub struct StepArg {
    name: &'static str,
    ty: &'static str,
    value_fn: fn() -> String,
}

impl StepArg {
    /// This is intended to be used by narrative-macros
    pub fn new(name: &'static str, ty: &'static str, value_fn: fn() -> String) -> Self {
        Self { name, ty, value_fn }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn ty(&self) -> &'static str {
        self.ty
    }

    /// Returns the debug representation of the value.
    pub fn value(&self) -> String {
        (self.value_fn)()
    }
}
