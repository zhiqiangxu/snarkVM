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

use super::*;

impl<N: Network> Store<N> for FinalizeRegisters<N> {
    /// Assigns the given value to the given register, assuming the register is not already assigned.
    ///
    /// # Errors
    /// This method will halt if the given register is a register member.
    /// This method will halt if the given register is an input register.
    /// This method will halt if the register is already used.
    #[inline]
    fn store(&mut self, stack: &Stack<N>, register: &Register<N>, stack_value: Value<N>) -> Result<()> {
        // Ensure that the stack value is a plaintext value.
        let plaintext_value = match stack_value {
            Value::Plaintext(plaintext) => plaintext,
            Value::Record(_) => bail!("Cannot store a record in a finalize register"),
        };
        // Store the value to the register.
        match register {
            Register::Locator(locator) => {
                // Ensure the register assignments are monotonically increasing.
                let expected_locator = self.registers.len() as u64;
                ensure!(expected_locator == *locator, "Out-of-order write operation at '{register}'");
                // Ensure the register does not already exist.
                ensure!(!self.registers.contains_key(locator), "Cannot write to occupied register '{register}'");

                // Ensure the type of the register is valid.
                match self.finalize_types.get_type(stack, register) {
                    // Ensure the plaintext value matches the plaintext type.
                    Ok(plaintext_type) => stack.matches_plaintext(&plaintext_value, &plaintext_type)?,
                    // Ensure the register is defined.
                    Err(error) => bail!("Register '{register}' is missing a type definition: {error}"),
                };

                // Store the plaintext value.
                match self.registers.insert(*locator, plaintext_value) {
                    // Ensure the register has not been previously stored.
                    Some(..) => bail!("Attempted to write to register '{register}' again"),
                    // Return on success.
                    None => Ok(()),
                }
            }
            // Ensure the register is not a register member.
            Register::Member(..) => bail!("Cannot store to a register member: '{register}'"),
        }
    }
}
