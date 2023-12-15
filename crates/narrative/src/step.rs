#[derive(Clone, Copy)]
pub struct Step<T, E> {
    index: usize,
    step_fn: fn(&mut T) -> Result<(), E>,
    step_text: fn() -> String,
    args: Args,
}

impl<T, E> Step<T, E> {
    pub fn run(&self, story: &mut T) -> Result<(), E> {
        (self.step_fn)(story)
    }

    pub fn step_text(&self) -> String {
        (self.step_text)()
    }

    pub fn args(&self) -> Args {
        self.args
    }

    pub fn id(&self) -> StepId<T> {
        StepId {
            index: self.index,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Args {
    args: &'static [StepArg],
}

impl Args {
    pub fn iter(&self) -> std::slice::Iter<'static, StepArg> {
        self.args.iter()
    }
}

impl IntoIterator for Args {
    type Item = &'static StepArg;
    type IntoIter = std::slice::Iter<'static, StepArg>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

pub struct StepArg {
    name: &'static str,
    ty: &'static str,
    value: fn() -> String,
}

impl StepArg {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn ty(&self) -> &'static str {
        self.ty
    }

    /// Returns the debug representation of the value.
    pub fn value(&self) -> String {
        (self.value)()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The identifier of a step. It can be used to find the corresponding step.
pub struct StepId<T> {
    // This must not be public to prevent the user from creating an invalid StepId.
    index: usize,
    // It ensures that the index is valid in the story of type T.
    _phantom: std::marker::PhantomData<T>,
}

impl<T> StepId<T> {
    /// Returns the index of the step.
    pub fn index(&self) -> usize {
        self.index
    }
}

pub struct Steps<T: 'static, E: 'static> {
    steps: &'static [Step<T, E>],
}

impl<T: 'static, E: 'static> Steps<T, E> {
    pub fn get(&self, id: StepId<T>) -> &'static Step<T, E> {
        // This never panics unless narrative implementation is incorrect.
        &self.steps[id.index]
    }

    pub fn iter(&self) -> std::slice::Iter<'static, Step<T, E>> {
        self.steps.iter()
    }
}

impl<T: 'static, E: 'static> IntoIterator for Steps<T, E> {
    type Item = &'static Step<T, E>;
    type IntoIter = std::slice::Iter<'static, Step<T, E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}
