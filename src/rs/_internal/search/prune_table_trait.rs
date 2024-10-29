use std::sync::Arc;

use crate::_internal::puzzle_traits::SemiGroupActionPuzzle;

use super::{index_type, IDFSearchAPIData, SearchLogger};

index_type!(Depth);

pub trait PruneTable<TPuzzle: SemiGroupActionPuzzle> {
    // TODO: design a proper API. The args here are currently inherited from `HashPruneTable`
    fn new(
        tpuzzle: TPuzzle,
        search_api_data: Arc<IDFSearchAPIData<TPuzzle>>,
        search_logger: Arc<SearchLogger>,
        min_size: Option<usize>,
    ) -> Self;

    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth;

    // TODO
    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize);
}
