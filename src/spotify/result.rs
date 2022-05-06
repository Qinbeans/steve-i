pub enum QueryType{
    Playlist,
    Song
}

pub struct QueryResult{
    pub result: String,
    pub tp: QueryType
}
