#[derive(Clone, Debug, PartialEq)]
pub enum TaskFilter {
    ActiveOnly,
    All,
    CompletedOnly,
}
