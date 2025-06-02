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

//! Collectible Fungible Assets (CFA) schema.

use aluvm::library::LibSite;
use amplify::confinement::Confined;
use rgbstd::contract::{
    AssignmentsFilter, ContractData, FungibleAllocation, IssuerWrapper, SchemaWrapper,
};
use rgbstd::persistence::ContractStateRead;
use rgbstd::schema::{
    AssignmentDetails, FungibleType, GenesisSchema, GlobalDetails, GlobalStateSchema, Occurrences,
    Schema, TransitionDetails, TransitionSchema,
};
use rgbstd::stl::{rgb_contract_stl, ContractTerms, Details, Name, StandardTypes};
use rgbstd::validation::Scripts;
use rgbstd::{Amount, OwnedStateSchema, Precision, SchemaId};
use strict_types::TypeSystem;

use crate::nia::{nia_lib, FN_NIA_GENESIS_OFFSET, FN_NIA_TRANSFER_OFFSET};
use crate::{
    GS_ART, GS_DETAILS, GS_ISSUED_SUPPLY, GS_NAME, GS_PRECISION, GS_TERMS, OS_ASSET, TS_TRANSFER,
};

pub const CFA_SCHEMA_ID: SchemaId = SchemaId::from_array([
    0x26, 0x0a, 0x8a, 0xe6, 0x12, 0x57, 0xf5, 0x80, 0x53, 0xe2, 0x8b, 0x02, 0x57, 0xb5, 0x5c, 0x5b,
    0xe8, 0x8b, 0x4d, 0xc0, 0x39, 0x72, 0xc5, 0x02, 0x9c, 0xbc, 0xef, 0x68, 0xa4, 0xd3, 0xac, 0xd6,
]);

fn cfa_standard_types() -> StandardTypes { StandardTypes::with(rgb_contract_stl()) }

pub fn cfa_schema() -> Schema {
    let types = cfa_standard_types();

    let nia_id = nia_lib().id();

    Schema {
        ffv: zero!(),
        name: tn!("CollectibleFungibleAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_ART => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Article")),
                name: fname!("art"),
            },
            GS_NAME => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Name")),
                name: fname!("name"),
            },
            GS_DETAILS => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Details")),
                name: fname!("details"),
            },
            GS_PRECISION => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Precision")),
                name: fname!("precision"),
            },
            GS_TERMS => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.ContractTerms")),
                name: fname!("terms"),
            },
            GS_ISSUED_SUPPLY => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Amount")),
                name: fname!("issuedSupply"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            }
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_ART => Occurrences::NoneOrOnce,
                GS_NAME => Occurrences::Once,
                GS_DETAILS => Occurrences::NoneOrOnce,
                GS_PRECISION => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: Some(LibSite::with(FN_NIA_GENESIS_OFFSET, nia_id)),
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::OnceOrMore
                    },
                    assignments: tiny_bmap! {
                        OS_ASSET => Occurrences::OnceOrMore
                    },
                    validator: Some(LibSite::with(FN_NIA_TRANSFER_OFFSET, nia_id))
                },
                name: fname!("transfer"),
            }
        },
        default_assignment: Some(OS_ASSET),
    }
}

#[derive(Default)]
pub struct CollectibleFungibleAsset;

#[derive(Clone, Eq, PartialEq, Debug, From)]
pub struct CfaWrapper<S: ContractStateRead>(ContractData<S>);

impl IssuerWrapper for CollectibleFungibleAsset {
    type Wrapper<S: ContractStateRead> = CfaWrapper<S>;

    fn schema() -> Schema { cfa_schema() }

    fn types() -> TypeSystem { cfa_standard_types().type_system(cfa_schema()) }

    fn scripts() -> Scripts {
        let lib = nia_lib();
        Confined::from_checked(bmap! { lib.id() => lib })
    }
}

impl<S: ContractStateRead> SchemaWrapper<S> for CfaWrapper<S> {
    fn with(data: ContractData<S>) -> Self {
        if data.schema.schema_id() != CFA_SCHEMA_ID {
            panic!("the provided schema is not CFA");
        }
        Self(data)
    }
}

impl<S: ContractStateRead> CfaWrapper<S> {
    pub fn name(&self) -> Name {
        let strict_val = &self
            .0
            .global("name")
            .next()
            .expect("CFA requires global state `name` to have at least one item");
        Name::from_strict_val_unchecked(strict_val)
    }

    pub fn details(&self) -> Option<Details> {
        self.0
            .global("details")
            .next()
            .map(|strict_val| Details::from_strict_val_unchecked(&strict_val))
    }

    pub fn precision(&self) -> Precision {
        let strict_val = &self
            .0
            .global("precision")
            .next()
            .expect("CFA requires global state `precision` to have at least one item");
        Precision::from_strict_val_unchecked(strict_val)
    }

    pub fn total_issued_supply(&self) -> Amount {
        self.0
            .global("issuedSupply")
            .map(|amount| Amount::from_strict_val_unchecked(&amount))
            .sum()
    }

    pub fn contract_terms(&self) -> ContractTerms {
        let strict_val = &self
            .0
            .global("terms")
            .next()
            .expect("CFA requires global state `terms` to have at least one item");
        ContractTerms::from_strict_val_unchecked(strict_val)
    }

    pub fn allocations<'c>(
        &'c self,
        filter: impl AssignmentsFilter + 'c,
    ) -> impl Iterator<Item = FungibleAllocation> + 'c {
        self.0.fungible_raw(OS_ASSET, filter).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn schema_id() {
        let schema_id = cfa_schema().schema_id();
        eprintln!("{:#04x?}", schema_id.to_byte_array());
        assert_eq!(CFA_SCHEMA_ID, schema_id);
    }
}
