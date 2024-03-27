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

//! Non-Inflatable Assets (NIA) schema implementing RGB20 fungible assets
//! interface.

use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use bp::dbc::Method;
use rgbstd::containers::Contract;
use rgbstd::interface::rgb20::{AllocationError, PrimaryIssue};
use rgbstd::interface::{
    BuilderError, IfaceClass, IfaceImpl, IssuerClass, NamedField, Rgb20, TxOutpoint,
    VerNo,
};
use rgbstd::invoice::{Amount, Precision};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, Schema, Script, StateSchema,
    SubSchema, TransitionSchema,
};
use rgbstd::stl::{Attachment, StandardTypes};
use rgbstd::vm::opcodes::{INSTR_PCCS, INSTR_PCVS};
use rgbstd::vm::{AluLib, AluScript, EntryPoint, RgbIsa};
use rgbstd::{rgbasm, AssetTag, BlindingFactor, Types};
use strict_encoding::InvalidIdent;
use strict_types::{SemId, Ty};

use crate::{GS_TERMS, GS_ISSUED_SUPPLY, GS_NOMINAL, OS_ASSET, TS_TRANSFER};

fn nia_schema() -> SubSchema {
    let types = StandardTypes::with(Rgb20::stl());

    let code = rgbasm! {
        // SUBROUTINE 1: genesis validation
        // Before doing a check we need to put a string into first string register which will be
        // used as an error message in case of the check failure.
        put     s16[0],"the issued amounts do not match between the reported global state and confidential owned state";
        // Checking pedersen commitments against reported amount of issued assets present in the
        // global state.
        pccs    0x0FA0,0x07D2   ;
        // If the check succeeds we need to terminate the subroutine.
        ret                     ;
        // SUBROUTINE 2: transfer validation
        put     s16[0],"the sum of input amounts do not match the sum of the output amounts";
        // Checking that the sum of pedersen commitments in inputs is equal to the sum in outputs.
        pcvs    0x0FA0          ;
    };
    let alu_lib = AluLib(Lib::assemble::<Instr<RgbIsa>>(&code).unwrap());
    let alu_id = alu_lib.id();
    const FN_GENESIS_OFFSET: u16 = 0;
    const FN_TRANSFER_OFFSET: u16 = 6 + 5 + 1;
    assert_eq!(alu_lib.code.as_ref()[FN_GENESIS_OFFSET as usize + 6], INSTR_PCCS);
    assert_eq!(alu_lib.code.as_ref()[FN_TRANSFER_OFFSET as usize + 6], INSTR_PCVS);

    Schema {
        ffv: zero!(),
        flags: none!(),
        subset_of: None,
        types: Types::Strict(types.type_system()),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.AssetTerms")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => StateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            valencies: none!(),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
                metadata: Ty::<SemId>::UNIT.sem_id_unnamed(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                valencies: none!(),
            }
        },
        script: Script::AluVM(AluScript {
            libs: confined_bmap! { alu_id => alu_lib },
            entry_points: confined_bmap! {
                EntryPoint::ValidateGenesis => LibSite::with(FN_GENESIS_OFFSET, alu_id),
                EntryPoint::ValidateTransition(TS_TRANSFER) => LibSite::with(FN_TRANSFER_OFFSET, alu_id),
            },
        }),
    }
}

fn nia_rgb20() -> IfaceImpl {
    let schema = nia_schema();
    let iface = Rgb20::iface();

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        script: none!(),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_TERMS, fname!("terms")),
            NamedField::with(GS_ISSUED_SUPPLY, fname!("issuedSupply")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedField::with(TS_TRANSFER, fname!("transfer")),
        },
        extensions: none!(),
    }
}

pub struct NonInflatableAsset(PrimaryIssue);

impl IssuerClass for NonInflatableAsset {
    type IssuingIface = Rgb20;

    fn schema() -> SubSchema { nia_schema() }
    fn issue_impl() -> IfaceImpl { nia_rgb20() }
}

impl NonInflatableAsset {
    #[inline]
    pub fn testnet(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
    ) -> Result<Self, InvalidIdent> {
        PrimaryIssue::testnet::<Self>(ticker, name, details, precision).map(Self)
    }

    #[inline]
    pub fn testnet_det<C: IssuerClass>(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        precision: Precision,
        asset_tag: AssetTag,
    ) -> Result<Self, InvalidIdent> {
        PrimaryIssue::testnet_det::<Self>(ticker, name, details, precision, asset_tag)
            .map(Self)
    }

    #[inline]
    pub fn add_terms(
        mut self,
        contract: &str,
        media: Option<Attachment>,
    ) -> Result<Self, InvalidIdent> {
        self.0 = self.0.add_terms(contract, media)?;
        Ok(self)
    }

    #[inline]
    pub fn allocate<O: TxOutpoint>(
        mut self,
        method: Method,
        beneficiary: O,
        amount: Amount,
    ) -> Result<Self, AllocationError> {
        self.0 = self.0.allocate(method, beneficiary, amount)?;
        Ok(self)
    }

    #[inline]
    pub fn allocate_all<O: TxOutpoint>(
        mut self,
        method: Method,
        allocations: impl IntoIterator<Item = (O, Amount)>,
    ) -> Result<Self, AllocationError> {
        self.0 = self.0.allocate_all(method, allocations)?;
        Ok(self)
    }

    /// Add asset allocation in a deterministic way.
    #[inline]
    pub fn allocate_det<O: TxOutpoint>(
        mut self,
        method: Method,
        beneficiary: O,
        seal_blinding: u64,
        amount: Amount,
        amount_blinding: BlindingFactor,
    ) -> Result<Self, AllocationError> {
        self.0 =
            self.0
                .allocate_det(method, beneficiary, seal_blinding, amount, amount_blinding)?;
        Ok(self)
    }

    // TODO: implement when bulletproofs are supported
    /*
    #[inline]
    pub fn conceal_allocations(mut self) -> Self {

    }
     */

    #[inline]
    pub fn issue_contract(self) -> Result<Contract, BuilderError> { self.0.issue_contract() }

    pub fn issue_contract_det(self, timestamp: i64) -> Result<Contract, BuilderError> { self.0.issue_contract_det(timestamp) }
}
