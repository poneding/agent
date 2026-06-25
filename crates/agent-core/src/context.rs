#[derive(Clone, Debug, Default)]
pub struct ContextAssembler;

impl ContextAssembler {
    #[must_use]
    pub fn assemble(&self, user_input: &str) -> Vec<String> {
        vec![user_input.to_string()]
    }
}
