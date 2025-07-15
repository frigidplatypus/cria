#[derive(Clone, Debug)]
pub enum SortOrder {
    Default,
    TitleAZ,
    TitleZA,
    PriorityHighToLow,
    PriorityLowToHigh,
    FavoriteStarredFirst,
    DueDateEarliestFirst,
    DueDateLatestFirst,
    StartDateEarliestFirst,
    StartDateLatestFirst,
}
