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

use crate::Aleo;
use snarkvm_circuit_algorithms::{
    Commit,
    CommitUncompressed,
    Hash,
    HashMany,
    HashToGroup,
    HashToScalar,
    Pedersen128,
    Pedersen64,
    Poseidon2,
    Poseidon4,
    Poseidon8,
    BHP1024,
    BHP256,
    BHP512,
    BHP768,
};
use snarkvm_circuit_collections::merkle_tree::MerklePath;
use snarkvm_circuit_types::{
    environment::{prelude::*, Assignment, Circuit, R1CS},
    Boolean,
    Field,
    Group,
    Scalar,
};

use core::fmt;

type E = Circuit;

thread_local! {
    /// The group bases for the Aleo signature and encryption schemes.
    static GENERATOR_G: Vec<Group<AleoV0>> = Vec::constant(<console::Testnet3 as console::Network>::g_powers().to_vec());

    /// The balance commitment domain as a constant field element.
    static BCM_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::bcm_domain());
    /// The encryption domain as a constant field element.
    static ENCRYPTION_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::encryption_domain());
    /// The graph key domain as a constant field element.
    static GRAPH_KEY_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::graph_key_domain());
    /// The randomizer domain as a constant field element.
    static RANDOMIZER_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::randomizer_domain());
    /// The balance commitment randomizer domain as a constant field element.
    static R_BCM_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::r_bcm_domain());
    /// The serial number domain as a constant field element.
    static SERIAL_NUMBER_DOMAIN: Field<AleoV0> = Field::constant(<console::Testnet3 as console::Network>::serial_number_domain());

    /// The BHP hash function, which can take an input of up to 256 bits.
    static BHP_256: BHP256<AleoV0> = BHP256::<AleoV0>::constant(console::BHP_256.clone());
    /// The BHP hash function, which can take an input of up to 512 bits.
    static BHP_512: BHP512<AleoV0> = BHP512::<AleoV0>::constant(console::BHP_512.clone());
    /// The BHP hash function, which can take an input of up to 768 bits.
    static BHP_768: BHP768<AleoV0> = BHP768::<AleoV0>::constant(console::BHP_768.clone());
    /// The BHP hash function, which can take an input of up to 1024 bits.
    static BHP_1024: BHP1024<AleoV0> = BHP1024::<AleoV0>::constant(console::BHP_1024.clone());

    /// The Pedersen hash function, which can take an input of up to 64 bits.
    static PEDERSEN_64: Pedersen64<AleoV0> = Pedersen64::<AleoV0>::constant(console::PEDERSEN_64.clone());
    /// The Pedersen hash function, which can take an input of up to 128 bits.
    static PEDERSEN_128: Pedersen128<AleoV0> = Pedersen128::<AleoV0>::constant(console::PEDERSEN_128.clone());

    /// The Poseidon hash function, using a rate of 2.
    static POSEIDON_2: Poseidon2<AleoV0> = Poseidon2::<AleoV0>::constant(console::POSEIDON_2.clone());
    /// The Poseidon hash function, using a rate of 4.
    static POSEIDON_4: Poseidon4<AleoV0> = Poseidon4::<AleoV0>::constant(console::POSEIDON_4.clone());
    /// The Poseidon hash function, using a rate of 8.
    static POSEIDON_8: Poseidon8<AleoV0> = Poseidon8::<AleoV0>::constant(console::POSEIDON_8.clone());
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AleoV0;

impl Aleo for AleoV0 {
    /// Returns the balance commitment domain as a constant field element.
    fn bcm_domain() -> Field<Self> {
        BCM_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the encryption domain as a constant field element.
    fn encryption_domain() -> Field<Self> {
        ENCRYPTION_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the graph key domain as a constant field element.
    fn graph_key_domain() -> Field<Self> {
        GRAPH_KEY_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the randomizer domain as a constant field element.
    fn randomizer_domain() -> Field<Self> {
        RANDOMIZER_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the balance commitment randomizer domain as a constant field element.
    fn r_bcm_domain() -> Field<Self> {
        R_BCM_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the serial number domain as a constant field element.
    fn serial_number_domain() -> Field<Self> {
        SERIAL_NUMBER_DOMAIN.with(|domain| domain.clone())
    }

    /// Returns the scalar multiplication on the generator `G`.
    #[inline]
    fn g_scalar_multiply(scalar: &Scalar<Self>) -> Group<Self> {
        GENERATOR_G.with(|bases| {
            bases
                .iter()
                .zip_eq(&scalar.to_bits_le())
                .fold(Group::zero(), |output, (base, bit)| Group::ternary(bit, &(&output + base), &output))
        })
    }

    /// Returns a BHP commitment with an input hasher of 256-bits.
    fn commit_bhp256(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Field<Self> {
        BHP_256.with(|bhp| bhp.commit(input, randomizer))
    }

    /// Returns a BHP commitment with an input hasher of 512-bits.
    fn commit_bhp512(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Field<Self> {
        BHP_512.with(|bhp| bhp.commit(input, randomizer))
    }

    /// Returns a BHP commitment with an input hasher of 768-bits.
    fn commit_bhp768(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Field<Self> {
        BHP_768.with(|bhp| bhp.commit(input, randomizer))
    }

    /// Returns a BHP commitment with an input hasher of 1024-bits.
    fn commit_bhp1024(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Field<Self> {
        BHP_1024.with(|bhp| bhp.commit(input, randomizer))
    }

    /// Returns a Pedersen commitment for the given (up to) 64-bit input and randomizer.
    fn commit_ped64(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Group<Self> {
        PEDERSEN_64.with(|pedersen| pedersen.commit_uncompressed(input, randomizer))
    }

    /// Returns a Pedersen commitment for the given (up to) 128-bit input and randomizer.
    fn commit_ped128(input: &[Boolean<Self>], randomizer: &Scalar<Self>) -> Group<Self> {
        PEDERSEN_128.with(|pedersen| pedersen.commit_uncompressed(input, randomizer))
    }

    /// Returns the BHP hash with an input hasher of 256-bits.
    fn hash_bhp256(input: &[Boolean<Self>]) -> Field<Self> {
        BHP_256.with(|bhp| bhp.hash(input))
    }

    /// Returns the BHP hash with an input hasher of 512-bits.
    fn hash_bhp512(input: &[Boolean<Self>]) -> Field<Self> {
        BHP_512.with(|bhp| bhp.hash(input))
    }

    /// Returns the BHP hash with an input hasher of 768-bits.
    fn hash_bhp768(input: &[Boolean<Self>]) -> Field<Self> {
        BHP_768.with(|bhp| bhp.hash(input))
    }

    /// Returns the BHP hash with an input hasher of 1024-bits.
    fn hash_bhp1024(input: &[Boolean<Self>]) -> Field<Self> {
        BHP_1024.with(|bhp| bhp.hash(input))
    }

    /// Returns the Pedersen hash for a given (up to) 64-bit input.
    fn hash_ped64(input: &[Boolean<Self>]) -> Field<Self> {
        PEDERSEN_64.with(|pedersen| pedersen.hash(input))
    }

    /// Returns the Pedersen hash for a given (up to) 128-bit input.
    fn hash_ped128(input: &[Boolean<Self>]) -> Field<Self> {
        PEDERSEN_128.with(|pedersen| pedersen.hash(input))
    }

    /// Returns the Poseidon hash with an input rate of 2.
    fn hash_psd2(input: &[Field<Self>]) -> Field<Self> {
        POSEIDON_2.with(|poseidon| poseidon.hash(input))
    }

    /// Returns the Poseidon hash with an input rate of 4.
    fn hash_psd4(input: &[Field<Self>]) -> Field<Self> {
        POSEIDON_4.with(|poseidon| poseidon.hash(input))
    }

    /// Returns the Poseidon hash with an input rate of 8.
    fn hash_psd8(input: &[Field<Self>]) -> Field<Self> {
        POSEIDON_8.with(|poseidon| poseidon.hash(input))
    }

    /// Returns the extended Poseidon hash with an input rate of 2.
    fn hash_many_psd2(input: &[Field<Self>], num_outputs: u16) -> Vec<Field<Self>> {
        POSEIDON_2.with(|poseidon| poseidon.hash_many(input, num_outputs))
    }

    /// Returns the extended Poseidon hash with an input rate of 4.
    fn hash_many_psd4(input: &[Field<Self>], num_outputs: u16) -> Vec<Field<Self>> {
        POSEIDON_4.with(|poseidon| poseidon.hash_many(input, num_outputs))
    }

    /// Returns the extended Poseidon hash with an input rate of 8.
    fn hash_many_psd8(input: &[Field<Self>], num_outputs: u16) -> Vec<Field<Self>> {
        POSEIDON_8.with(|poseidon| poseidon.hash_many(input, num_outputs))
    }

    /// Returns the Poseidon hash with an input rate of 2 on the affine curve.
    fn hash_to_group_psd2(input: &[Field<Self>]) -> Group<Self> {
        POSEIDON_2.with(|poseidon| poseidon.hash_to_group(input))
    }

    /// Returns the Poseidon hash with an input rate of 4 on the affine curve.
    fn hash_to_group_psd4(input: &[Field<Self>]) -> Group<Self> {
        POSEIDON_4.with(|poseidon| poseidon.hash_to_group(input))
    }

    /// Returns the Poseidon hash with an input rate of 8 on the affine curve.
    fn hash_to_group_psd8(input: &[Field<Self>]) -> Group<Self> {
        POSEIDON_8.with(|poseidon| poseidon.hash_to_group(input))
    }

    /// Returns the Poseidon hash with an input rate of 2 on the scalar field.
    fn hash_to_scalar_psd2(input: &[Field<Self>]) -> Scalar<Self> {
        POSEIDON_2.with(|poseidon| poseidon.hash_to_scalar(input))
    }

    /// Returns the Poseidon hash with an input rate of 4 on the scalar field.
    fn hash_to_scalar_psd4(input: &[Field<Self>]) -> Scalar<Self> {
        POSEIDON_4.with(|poseidon| poseidon.hash_to_scalar(input))
    }

    /// Returns the Poseidon hash with an input rate of 8 on the scalar field.
    fn hash_to_scalar_psd8(input: &[Field<Self>]) -> Scalar<Self> {
        POSEIDON_8.with(|poseidon| poseidon.hash_to_scalar(input))
    }

    /// Returns `true` if the given Merkle path is valid for the given root and leaf.
    fn verify_merkle_path_bhp<const DEPTH: u8>(
        path: &MerklePath<Self, DEPTH>,
        root: &Field<Self>,
        leaf: &Vec<Boolean<Self>>,
    ) -> Boolean<Self> {
        BHP_1024.with(|bhp1024| BHP_512.with(|bhp512| path.verify(bhp1024, bhp512, root, leaf)))
    }

    /// Returns `true` if the given Merkle path is valid for the given root and leaf.
    fn verify_merkle_path_psd<const DEPTH: u8>(
        path: &MerklePath<Self, DEPTH>,
        root: &Field<Self>,
        leaf: &Vec<Field<Self>>,
    ) -> Boolean<Self> {
        POSEIDON_4.with(|psd4| POSEIDON_2.with(|psd2| path.verify(psd4, psd2, root, leaf)))
    }
}

impl Environment for AleoV0 {
    type Affine = <E as Environment>::Affine;
    type BaseField = <E as Environment>::BaseField;
    type Network = <E as Environment>::Network;
    type ScalarField = <E as Environment>::ScalarField;

    /// Returns the `zero` constant.
    fn zero() -> LinearCombination<Self::BaseField> {
        E::zero()
    }

    /// Returns the `one` constant.
    fn one() -> LinearCombination<Self::BaseField> {
        E::one()
    }

    /// Returns a new variable of the given mode and value.
    fn new_variable(mode: Mode, value: Self::BaseField) -> Variable<Self::BaseField> {
        E::new_variable(mode, value)
    }

    /// Returns a new witness of the given mode and value.
    fn new_witness<Fn: FnOnce() -> Output::Primitive, Output: Inject>(mode: Mode, logic: Fn) -> Output {
        E::new_witness(mode, logic)
    }

    /// Enters a new scope for the environment.
    fn scope<S: Into<String>, Fn, Output>(name: S, logic: Fn) -> Output
    where
        Fn: FnOnce() -> Output,
    {
        E::scope(name, logic)
    }

    /// Adds one constraint enforcing that `(A * B) == C`.
    fn enforce<Fn, A, B, C>(constraint: Fn)
    where
        Fn: FnOnce() -> (A, B, C),
        A: Into<LinearCombination<Self::BaseField>>,
        B: Into<LinearCombination<Self::BaseField>>,
        C: Into<LinearCombination<Self::BaseField>>,
    {
        E::enforce(constraint)
    }

    /// Returns `true` if all constraints in the environment are satisfied.
    fn is_satisfied() -> bool {
        E::is_satisfied()
    }

    /// Returns `true` if all constraints in the current scope are satisfied.
    fn is_satisfied_in_scope() -> bool {
        E::is_satisfied_in_scope()
    }

    /// Returns the number of constants in the entire circuit.
    fn num_constants() -> u64 {
        E::num_constants()
    }

    /// Returns the number of public variables in the entire circuit.
    fn num_public() -> u64 {
        E::num_public()
    }

    /// Returns the number of private variables in the entire circuit.
    fn num_private() -> u64 {
        E::num_private()
    }

    /// Returns the number of constraints in the entire circuit.
    fn num_constraints() -> u64 {
        E::num_constraints()
    }

    /// Returns the number of gates in the entire circuit.
    fn num_gates() -> u64 {
        E::num_gates()
    }

    /// Returns the number of constants for the current scope.
    fn num_constants_in_scope() -> u64 {
        E::num_constants_in_scope()
    }

    /// Returns the number of public variables for the current scope.
    fn num_public_in_scope() -> u64 {
        E::num_public_in_scope()
    }

    /// Returns the number of private variables for the current scope.
    fn num_private_in_scope() -> u64 {
        E::num_private_in_scope()
    }

    /// Returns the number of constraints for the current scope.
    fn num_constraints_in_scope() -> u64 {
        E::num_constraints_in_scope()
    }

    /// Returns the number of gates for the current scope.
    fn num_gates_in_scope() -> u64 {
        E::num_gates_in_scope()
    }

    /// Halts the program from further synthesis, evaluation, and execution in the current environment.
    fn halt<S: Into<String>, T>(message: S) -> T {
        E::halt(message)
    }

    /// Returns the R1CS circuit, resetting the circuit.
    fn inject_r1cs(r1cs: R1CS<Self::BaseField>) {
        E::inject_r1cs(r1cs)
    }

    /// Returns the R1CS circuit, resetting the circuit.
    fn eject_r1cs_and_reset() -> R1CS<Self::BaseField> {
        E::eject_r1cs_and_reset()
    }

    /// Returns the R1CS assignment of the circuit, resetting the circuit.
    fn eject_assignment_and_reset() -> Assignment<<Self::Network as console::Environment>::Field> {
        E::eject_assignment_and_reset()
    }

    /// Clears the circuit and initializes an empty environment.
    fn reset() {
        E::reset()
    }
}

impl Display for AleoV0 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO (howardwu): Find a better way to print the circuit.
        fmt::Display::fmt(&Circuit, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_types::Field;

    type CurrentAleo = AleoV0;

    /// Compute 2^EXPONENT - 1, in a purposefully constraint-inefficient manner for testing.
    fn create_example_circuit<E: Environment>() -> Field<E> {
        let one = snarkvm_console_types::Field::<<E as Environment>::Network>::one();
        let two = one + one;

        const EXPONENT: u64 = 64;

        // Compute 2^EXPONENT - 1, in a purposefully constraint-inefficient manner for testing.
        let mut candidate = Field::<E>::new(Mode::Public, one);
        let mut accumulator = Field::new(Mode::Private, two);
        for _ in 0..EXPONENT {
            candidate += &accumulator;
            accumulator *= Field::new(Mode::Private, two);
        }

        assert_eq!((accumulator - Field::one()).eject_value(), candidate.eject_value());
        assert_eq!(2, E::num_public());
        assert_eq!(2 * EXPONENT + 1, E::num_private());
        assert_eq!(EXPONENT, E::num_constraints());
        assert!(E::is_satisfied());

        candidate
    }

    #[test]
    fn test_print_circuit() {
        let circuit = CurrentAleo {};
        let _candidate = create_example_circuit::<CurrentAleo>();
        let output = format!("{circuit}");
        println!("{output}");
    }

    #[test]
    fn test_circuit_scope() {
        CurrentAleo::scope("test_circuit_scope", || {
            assert_eq!(0, CurrentAleo::num_constants());
            assert_eq!(1, CurrentAleo::num_public());
            assert_eq!(0, CurrentAleo::num_private());
            assert_eq!(0, CurrentAleo::num_constraints());

            assert_eq!(0, CurrentAleo::num_constants_in_scope());
            assert_eq!(0, CurrentAleo::num_public_in_scope());
            assert_eq!(0, CurrentAleo::num_private_in_scope());
            assert_eq!(0, CurrentAleo::num_constraints_in_scope());
        })
    }
}
