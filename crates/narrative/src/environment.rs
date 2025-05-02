/// A dummy environment that does nothing on each step. Every story can be run on this environment.
pub struct DummyEnvironment<E>(std::marker::PhantomData<E>);

impl<E> Default for DummyEnvironment<E> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<E> std::fmt::Debug for DummyEnvironment<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DummyEnvironment").finish()
    }
}
