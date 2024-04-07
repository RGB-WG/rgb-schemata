// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2023-2024 LNP/BP Standards Association. All rights reserved.
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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

mod cfa;
mod nia;
mod uda;

pub use cfa::{cfa_rgb25, cfa_schema};
pub use nia::NonInflatableAsset;
use rgbstd::{AssignmentType, GlobalStateType, TransitionType};
pub use uda::{uda_rgb21, uda_schema};

pub const GS_NOMINAL: GlobalStateType = GlobalStateType::with(2000);
pub const GS_TERMS: GlobalStateType = GlobalStateType::with(2001);
pub const GS_ISSUED_SUPPLY: GlobalStateType = GlobalStateType::with(2002);
pub const OS_ASSET: AssignmentType = AssignmentType::with(4000);
pub const TS_TRANSFER: TransitionType = TransitionType::with(10000);

pub mod dumb {
    use std::convert::Infallible;

    use rgbstd::resolvers::ResolveHeight;
    use rgbstd::validation::{ResolveWitness, WitnessResolverError};
    use rgbstd::{WitnessAnchor, XWitnessId, XWitnessTx};
    use strict_encoding::StrictDumb;

    pub struct DumbResolver;

    impl ResolveWitness for DumbResolver {
        fn resolve_pub_witness(&self, _: XWitnessId) -> Result<XWitnessTx, WitnessResolverError> {
            Ok(XWitnessTx::strict_dumb())
        }
    }

    impl ResolveHeight for DumbResolver {
        type Error = Infallible;
        fn resolve_height(&mut self, _: &XWitnessId) -> Result<WitnessAnchor, Self::Error> {
            Ok(WitnessAnchor::strict_dumb())
        }
    }
}
