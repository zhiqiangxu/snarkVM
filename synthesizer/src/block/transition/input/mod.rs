// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

mod bytes;
mod serialize;
mod string;

use console::{
    network::prelude::*,
    program::{Ciphertext, Plaintext, TransitionLeaf},
    types::Field,
};

type Variant = u8;

/// The transition input.
#[derive(Clone, PartialEq, Eq)]
pub enum Input<N: Network> {
    /// The plaintext hash and (optional) plaintext.
    Constant(Field<N>, Option<Plaintext<N>>),
    /// The plaintext hash and (optional) plaintext.
    Public(Field<N>, Option<Plaintext<N>>),
    /// The ciphertext hash and (optional) ciphertext.
    Private(Field<N>, Option<Ciphertext<N>>),
    /// The serial number and tag of the record.
    Record(Field<N>, Field<N>),
    /// The input commitment to the external record. Note: This is **not** the record commitment.
    ExternalRecord(Field<N>),
}

impl<N: Network> Input<N> {
    /// Returns the variant of the input.
    pub const fn variant(&self) -> Variant {
        match self {
            Input::Constant(..) => 0,
            Input::Public(..) => 1,
            Input::Private(..) => 2,
            Input::Record(..) => 3, // <- Changing this will invalidate 'console::StatePath' and 'circuit::StatePath'.
            Input::ExternalRecord(..) => 4,
        }
    }

    /// Returns the ID of the input.
    pub const fn id(&self) -> &Field<N> {
        match self {
            Input::Constant(id, ..) => id,
            Input::Public(id, ..) => id,
            Input::Private(id, ..) => id,
            Input::Record(serial_number, ..) => serial_number,
            Input::ExternalRecord(id) => id,
        }
    }

    /// Returns the input as a transition leaf.
    pub fn to_transition_leaf(&self, index: u8) -> TransitionLeaf<N> {
        TransitionLeaf::new_with_version(index, self.variant(), *self.id())
    }

    /// Returns the tag, if the input is a record.
    pub const fn tag(&self) -> Option<&Field<N>> {
        match self {
            Input::Record(_, tag) => Some(tag),
            _ => None,
        }
    }

    /// Returns the tag, if the input is a record, and consumes `self`.
    pub fn into_tag(self) -> Option<Field<N>> {
        match self {
            Input::Record(_, tag) => Some(tag),
            _ => None,
        }
    }

    /// Returns the serial number, if the input is a record.
    pub const fn serial_number(&self) -> Option<&Field<N>> {
        match self {
            Input::Record(serial_number, ..) => Some(serial_number),
            _ => None,
        }
    }

    /// Returns the serial number, if the input is a record, and consumes `self`.
    pub fn into_serial_number(self) -> Option<Field<N>> {
        match self {
            Input::Record(serial_number, ..) => Some(serial_number),
            _ => None,
        }
    }

    /// Returns the public verifier inputs for the proof.
    pub fn verifier_inputs(&self) -> impl '_ + Iterator<Item = N::Field> {
        [Some(self.id()), self.tag()].into_iter().flatten().map(|id| **id)
    }

    /// Returns `true` if the input is well-formed.
    /// If the optional value exists, this method checks that it hashes to the input ID.
    pub fn verify(&self, function_id: Field<N>, tcm: &Field<N>, index: usize) -> bool {
        // Ensure the hash of the value (if the value exists) is correct.
        let result = || match self {
            Input::Constant(hash, Some(input)) => {
                match input.to_fields() {
                    Ok(fields) => {
                        // Construct the (console) input index as a field element.
                        let index = Field::from_u16(index as u16);
                        // Construct the preimage as `(function ID || input || tcm || index)`.
                        let mut preimage = vec![function_id];
                        preimage.extend(fields);
                        preimage.push(*tcm);
                        preimage.push(index);
                        // Ensure the hash matches.
                        match N::hash_psd8(&preimage) {
                            Ok(candidate_hash) => Ok(hash == &candidate_hash),
                            Err(error) => Err(error),
                        }
                    }
                    Err(error) => Err(error),
                }
            }
            Input::Public(hash, Some(input)) => {
                match input.to_fields() {
                    Ok(fields) => {
                        // Construct the (console) input index as a field element.
                        let index = Field::from_u16(index as u16);
                        // Construct the preimage as `(function ID || input || tcm || index)`.
                        let mut preimage = vec![function_id];
                        preimage.extend(fields);
                        preimage.push(*tcm);
                        preimage.push(index);
                        // Ensure the hash matches.
                        match N::hash_psd8(&preimage) {
                            Ok(candidate_hash) => Ok(hash == &candidate_hash),
                            Err(error) => Err(error),
                        }
                    }
                    Err(error) => Err(error),
                }
            }
            Input::Private(hash, Some(value)) => {
                match value.to_fields() {
                    // Ensure the hash matches.
                    Ok(fields) => match N::hash_psd8(&fields) {
                        Ok(candidate_hash) => Ok(hash == &candidate_hash),
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                }
            }
            Input::Constant(_, None) | Input::Public(_, None) | Input::Private(_, None) => {
                // This enforces that the transition *must* contain the value for this transition input.
                // A similar rule is enforced for the transition output.
                bail!("A transition input value is missing")
            }
            Input::Record(_, _) | Input::ExternalRecord(_) => Ok(true),
        };

        match result() {
            Ok(is_hash_valid) => is_hash_valid,
            Err(error) => {
                eprintln!("{error}");
                false
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use console::{network::Testnet3, program::Literal};

    type CurrentNetwork = Testnet3;

    /// Sample the transition inputs.
    pub(crate) fn sample_inputs() -> Vec<(<CurrentNetwork as Network>::TransitionID, Input<CurrentNetwork>)> {
        let rng = &mut TestRng::default();

        // Sample a transition.
        let transaction = crate::vm::test_helpers::sample_execution_transaction_with_fee(rng);
        let transition = transaction.transitions().next().unwrap();

        // Retrieve the transition ID and input.
        let transition_id = *transition.id();
        let input = transition.inputs().iter().next().unwrap().clone();

        // Sample a random plaintext.
        let plaintext = Plaintext::Literal(Literal::Field(Uniform::rand(rng)), Default::default());
        let plaintext_hash = CurrentNetwork::hash_bhp1024(&plaintext.to_bits_le()).unwrap();
        // Sample a random ciphertext.
        let ciphertext = Ciphertext::from_fields(&vec![Uniform::rand(rng); 10]).unwrap();
        let ciphertext_hash = CurrentNetwork::hash_bhp1024(&ciphertext.to_bits_le()).unwrap();

        vec![
            (transition_id, input),
            (Uniform::rand(rng), Input::Constant(Uniform::rand(rng), None)),
            (Uniform::rand(rng), Input::Constant(plaintext_hash, Some(plaintext.clone()))),
            (Uniform::rand(rng), Input::Public(Uniform::rand(rng), None)),
            (Uniform::rand(rng), Input::Public(plaintext_hash, Some(plaintext))),
            (Uniform::rand(rng), Input::Private(Uniform::rand(rng), None)),
            (Uniform::rand(rng), Input::Private(ciphertext_hash, Some(ciphertext))),
            (Uniform::rand(rng), Input::Record(Uniform::rand(rng), Uniform::rand(rng))),
            (Uniform::rand(rng), Input::ExternalRecord(Uniform::rand(rng))),
        ]
    }
}
