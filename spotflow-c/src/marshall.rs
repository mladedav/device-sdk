pub(crate) trait Marshall {
    type Target;

    fn marshall(target: Self::Target) -> Self;
}
