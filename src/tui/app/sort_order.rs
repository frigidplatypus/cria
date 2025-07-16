#[derive(Clone, Debug)]
#[allow(dead_code)]
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
