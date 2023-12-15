pub struct Narration {
    pub story_title: String,
    pub story_ident: &'static str,
    pub steps: Vec<NarrationStep>,
}

pub struct NarrationStep {
    pub text: String,
    pub step_ident: &'static str,
    pub args: Vec<NarrationStepArg>,
}

pub struct NarrationStepArg {
    pub ident: &'static str,
    pub ty: &'static str,
    pub value: String,
}
