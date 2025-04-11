use std::fmt::Debug;

use cubing::kpuzzle::{KPattern, KPuzzle};
use multiset::HashMultiSet;

use crate::_internal::{errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle};

pub trait HasTargetPatternSignature: SemiGroupActionPuzzle {
    type Signature: PartialEq + Eq + Debug;

    fn get_target_pattern_signature(pattern: &Self::Pattern) -> Self::Signature;
}

#[derive(PartialEq, Eq, Debug)]
pub struct KPatternOrbitPieceCountSignature {
    orbits: Vec<HashMultiSet<u8>>,
}

impl From<&KPattern> for KPatternOrbitPieceCountSignature {
    fn from(kpattern: &KPattern) -> Self {
        let orbits = vec![];
        for orbit_info in kpattern.kpuzzle().orbit_info_iter() {
            let mut orbit_pieces = HashMultiSet::<u8>::new();
            for i in 0..orbit_info.num_pieces {
                orbit_pieces.insert(kpattern.get_piece(orbit_info, i));
            }
        }
        Self { orbits }
    }
}

impl HasTargetPatternSignature for KPuzzle {
    type Signature = KPatternOrbitPieceCountSignature;

    fn get_target_pattern_signature(pattern: &Self::Pattern) -> Self::Signature {
        KPatternOrbitPieceCountSignature::from(pattern)
    }
}

// TODO: make this more reusable.
pub fn check_target_pattern_consistency<
    TPuzzle: SemiGroupActionPuzzle + HasTargetPatternSignature,
>(
    reference_pattern: &TPuzzle::Pattern,
    target_patterns: &mut dyn Iterator<Item = &TPuzzle::Pattern>,
) -> Result<(), SearchError> {
    // TODO: Push this further down into `try_new_prune_table_construction_shim` so it's used for 4x4x4
    let reference_pattern_orbit_signature =
        TPuzzle::get_target_pattern_signature(reference_pattern);
    for (i, target_pattern) in target_patterns.enumerate() {
        if reference_pattern_orbit_signature
            != TPuzzle::get_target_pattern_signature(target_pattern)
        {
            return Err(SearchError { description: format!("Orbit piece count signature for the reference pattern does not match the pattern at index {}.", i) });
        }
    }
    Ok(())
}

// TODO: make this more reusable.
pub fn check_target_pattern_consistency_single_iter<
    TPuzzle: SemiGroupActionPuzzle + HasTargetPatternSignature,
>(
    mut target_patterns: &mut dyn Iterator<Item = &TPuzzle::Pattern>,
) -> Result<(), SearchError> {
    let Some(first) = target_patterns.next() else {
        return Ok(());
    };
    check_target_pattern_consistency::<TPuzzle>(first, &mut target_patterns)
}
