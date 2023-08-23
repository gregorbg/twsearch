use std::{alloc::Layout, sync::Arc};

use cubing::{
    alg::Move,
    kpuzzle::{InvalidAlgError, InvalidDefinitionError, KPuzzle, KPuzzleOrbitName},
};

use super::{PackedKState, PackedKTransformation};

// TODO: index divisors
const MAX_NUM_ORIENTATIONS: usize = if cfg!(no_orientation_mod) { 127 } else { 16 };
#[cfg(not(feature = "no_orientation_mod"))]
pub const ORIENTATION_MOD_SHIFT_BITS: usize = 4;
#[cfg(not(feature = "no_orientation_mod"))]
pub const ORIENTATION_MASK: u8 = 0xF;

#[derive(Debug, Clone)]
pub struct PackedKPuzzleOrbitInfo {
    pub name: KPuzzleOrbitName,
    pub pieces_or_pemutations_offset: usize,
    pub orientations_offset: usize,
    pub num_pieces: usize,
    pub num_orientations: u8,
    #[cfg(feature = "no_orientation_mod")]
    pub unknown_orientation_value: u8,
}

#[derive(Debug, Clone)]
pub struct PackedKPuzzleData {
    pub kpuzzle: KPuzzle,
    // Private cached values.
    pub num_bytes: usize,
    pub orbit_iteration_info: Vec<PackedKPuzzleOrbitInfo>,
    pub layout: Layout,
}

#[derive(Debug, Clone)]
pub struct PackedKPuzzle {
    pub data: Arc<PackedKPuzzleData>, // TODO
                                      // pub data: PackedKPuzzleData,
}

impl TryFrom<KPuzzle> for PackedKPuzzle {
    type Error = InvalidDefinitionError;

    fn try_from(kpuzzle: KPuzzle) -> Result<Self, Self::Error> {
        let def = kpuzzle.definition();
        let orbit_ordering = &def.orbit_ordering;
        let orbit_ordering = orbit_ordering.as_ref().ok_or_else(|| InvalidDefinitionError{ description: "Constructing a `PackedKPuzzle` from a `KPuzzle` requires the `orbitOrdering` field.".to_owned()})?;

        let mut bytes_offset = 0;
        let mut orbit_iteration_info: Vec<PackedKPuzzleOrbitInfo> = vec![];

        for orbit_name in orbit_ordering {
            let orbit_definition = kpuzzle.definition().orbits.get(orbit_name);
            let orbit_definition = orbit_definition.ok_or_else(|| InvalidDefinitionError {
                description: format!(
                    "Missing orbit definition for orbit in ordering: {}",
                    orbit_name
                ),
            })?;
            let num_orientations = orbit_definition.num_orientations;
            if num_orientations > MAX_NUM_ORIENTATIONS {
                return Err(InvalidDefinitionError { description: format!("`num_orientations` for orbit {} is too large ({}). Maximum is {} for the current build." , orbit_name, num_orientations, MAX_NUM_ORIENTATIONS)});
            }
            orbit_iteration_info.push({
                PackedKPuzzleOrbitInfo {
                    name: orbit_name.clone(),
                    num_pieces: orbit_definition.num_pieces,
                    num_orientations: usize_to_u8(num_orientations),
                    pieces_or_pemutations_offset: bytes_offset,
                    orientations_offset: bytes_offset
                        + std::convert::Into::<usize>::into(orbit_definition.num_pieces),
                    #[cfg(feature = "no_orientation_mod")]
                    unknown_orientation_value: usize_to_u8(2 * num_orientations),
                }
            });
            bytes_offset += orbit_definition.num_pieces * 2;
        }

        Ok(Self {
            data: Arc::new(PackedKPuzzleData {
                kpuzzle,
                num_bytes: bytes_offset,
                orbit_iteration_info,
                layout: Layout::array::<u8>(bytes_offset).map_err(|_| InvalidDefinitionError {
                    description: "Could not construct packed layout.".to_owned(),
                })?,
            }),
        })
    }
}

/// An error type that can indicate multiple error causes, when parsing and applying an alg at the same time.
#[derive(derive_more::From, Debug, derive_more::Display)]
pub enum ConversionError {
    InvalidAlg(InvalidAlgError),
    InvalidDefinition(InvalidDefinitionError),
}

fn usize_to_u8(n: usize) -> u8 {
    n.try_into().expect("Value too large!") // TODO
}

impl PackedKPuzzle {
    pub fn start_state(&self) -> PackedKState {
        let kstate_start_state_data = self.data.kpuzzle.start_state().state_data;

        let new_state = PackedKState::new(self.clone());
        for orbit_info in &self.data.orbit_iteration_info {
            let kstate_orbit_data = kstate_start_state_data
                .get(&orbit_info.name)
                .expect("Missing orbit!");
            for i in 0..orbit_info.num_pieces {
                new_state.set_piece_or_permutation(
                    orbit_info,
                    i,
                    usize_to_u8(kstate_orbit_data.pieces[i]),
                );
                new_state.set_orientation(
                    orbit_info,
                    i,
                    match &kstate_orbit_data.orientation_mod {
                        None => usize_to_u8(kstate_orbit_data.orientation[i]),
                        Some(orientation_mod) => {
                            #[cfg(not(feature = "no_orientation_mod"))]
                            {
                                (usize_to_u8(orientation_mod[i]) << ORIENTATION_MOD_SHIFT_BITS)
                                    + usize_to_u8(kstate_orbit_data.orientation[i])
                            }
                            #[cfg(feature = "no_orientation_mod")]
                            {
                                match orientation_mod[i] {
                                    0 => usize_to_u8(kstate_orbit_data.orientation[i]),
                                    1 => orbit_info.unknown_orientation_value, // TODO
                                    _ => panic!("Unsupported!"),               // TODO
                                }
                            }
                        }
                    },
                );
            }
        }

        new_state
    }

    // TODO: implement this as a `TryFrom`?
    pub fn transformation_from_move(
        &self, // TODO: Any issues with not using `&self`?
        key_move: &Move,
    ) -> Result<PackedKTransformation, ConversionError> {
        let unpacked_ktransformation = self.data.kpuzzle.transformation_from_move(key_move)?;

        let new_transformation = PackedKTransformation::new(self.clone());
        for orbit_info in &self.data.orbit_iteration_info {
            let unpacked_orbit_data = unpacked_ktransformation
                .transformation_data
                .get(&orbit_info.name);
            let unpacked_orbit_data =
                unpacked_orbit_data.ok_or_else(|| InvalidDefinitionError {
                    description: format!("Missing orbit: {}", orbit_info.name),
                })?;
            for i in 0..orbit_info.num_pieces {
                new_transformation.set_piece_or_permutation(
                    orbit_info,
                    i,
                    usize_to_u8(unpacked_orbit_data.permutation[i]),
                );
                new_transformation.set_orientation(
                    orbit_info,
                    i,
                    usize_to_u8(unpacked_orbit_data.orientation[i]),
                )
            }
        }

        Ok(new_transformation)
    }
}
