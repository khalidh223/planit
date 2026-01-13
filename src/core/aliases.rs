use std::collections::HashMap;

pub type SourceId = i32;
pub type ResolvedId = i32;
pub type IdLookup = HashMap<SourceId, ResolvedId>;
pub type TokenList = Vec<String>;
pub type TokenMatrix = Vec<TokenList>;
