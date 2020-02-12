// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of `MallocSize` for common ethereum types: fixed hashes
//! and uints.

use ethereum_types::{Bloom, H128, H264, H32, H520, H64, U64};

malloc_size_of_is_0!(U64, H32, H64, H128, H264, H520, Bloom);
