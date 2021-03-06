// Copyright 2018 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Lightweight readonly view into output MMR for convenience.

use core::core::pmmr::ReadonlyPMMR;
use core::core::{Block, Input, Output, Transaction};

use error::{Error, ErrorKind};
use grin_store::pmmr::PMMRBackend;
use store::Batch;

/// Readonly view of the UTXO set (based on output MMR).
pub struct UTXOView<'a> {
	pmmr: ReadonlyPMMR<'a, Output, PMMRBackend<Output>>,
	batch: &'a Batch<'a>,
}

impl<'a> UTXOView<'a> {
	/// Build a new UTXO view.
	pub fn new(
		pmmr: ReadonlyPMMR<'a, Output, PMMRBackend<Output>>,
		batch: &'a Batch,
	) -> UTXOView<'a> {
		UTXOView { pmmr, batch }
	}

	/// Validate a block against the current UTXO set.
	/// Every input must spend an output that currently exists in the UTXO set.
	/// No duplicate outputs.
	pub fn validate_block(&self, block: &Block) -> Result<(), Error> {
		for output in block.outputs() {
			self.validate_output(output)?;
		}

		for input in block.inputs() {
			self.validate_input(input)?;
		}
		Ok(())
	}

	/// Validate a transaction against the current UTXO set.
	/// Every input must spend an output that currently exists in the UTXO set.
	/// No duplicate outputs.
	pub fn validate_tx(&self, tx: &Transaction) -> Result<(), Error> {
		for output in tx.outputs() {
			self.validate_output(output)?;
		}

		for input in tx.inputs() {
			self.validate_input(input)?;
		}
		Ok(())
	}

	fn validate_input(&self, input: &Input) -> Result<(), Error> {
		let commit = input.commitment();
		let pos_res = self.batch.get_output_pos(&commit);
		if let Ok(pos) = pos_res {
			if let Some(_) = self.pmmr.get_data(pos) {
				return Ok(());
			}
		}
		Err(ErrorKind::AlreadySpent(commit).into())
	}

	fn validate_output(&self, output: &Output) -> Result<(), Error> {
		let commit = output.commitment();
		if let Ok(pos) = self.batch.get_output_pos(&commit) {
			if let Some(out_mmr) = self.pmmr.get_data(pos) {
				if out_mmr.commitment() == commit {
					return Err(ErrorKind::DuplicateCommitment(commit).into());
				}
			}
		}
		Ok(())
	}
}
