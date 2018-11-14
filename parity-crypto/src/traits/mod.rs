// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.



//! trait module. Those traits expose current crypto usage in parity crates, event if they may evolve to
//! better prototype, that is not the current target.
//! The goal of those trait is to allow faster and more reliable switch of crypto dependencies.
//! It is done by running tests at a trait level.
//! Traits are only considering monomorphic usage (`dyn` usage is not covered).

pub mod asym;

