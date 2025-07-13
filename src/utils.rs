#[macro_export]
macro_rules! NO_IMPL {
    () => {
        eprintln!("NOT YET IMPLEMENTED");
        return
    };
}