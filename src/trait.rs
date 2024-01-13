trait ClonableFnMut<T, O>: FnMut(T) -> O {
    fn clone_box(&self) -> Box<dyn ClonableFnMut<T, O>>;
}

impl<F, T, O> ClonableFnMut<T, O> for F
where
    F: FnMut(T) -> O + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn ClonableFnMut<T, O>> {
        Box::new(self.clone())
    }
}