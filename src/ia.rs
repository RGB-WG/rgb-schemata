use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use ifaces::{rgb20, IssuerWrapper, Rgb20};
use rgbstd::interface::{IfaceClass, IfaceImpl, NamedField, NamedVariant, VerNo};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, OwnedStateSchema, Schema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::validation::Scripts;
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Identity};
use strict_types::TypeSystem;

use crate::{
    ERRNO_INFLATION_EXCEEDED_ALLOWANCE, ERRNO_INFLATION_MISMATCH, ERRNO_ISSUED_MISMATCH,
    ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY, GS_MAX_SUPPLY, GS_NOMINAL, GS_TERMS,
    MS_ALLOWED_INFLATION, OS_ASSET, OS_INFLATION_ALLOWANCE, TS_ISSUE, TS_TRANSFER,
};
pub(crate) const FN_IA_GENESIS_OFFSET: u16 = 4 + 3 + 2;
pub(crate) const FN_IA_TRANSFER_OFFSET: u16 = 0;

pub(crate) fn ia_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE Transfer validation
        // Set errno
        put     a8[0],ERRNO_NON_EQUAL_IN_OUT;
        // Checking that the sum of pedersen commitments in inputs is equal to the sum in outputs.
        pcvs    OS_ASSET;
        test;
        ret;

        // SUBROUTINE Genesis validation
        // Checking pedersen commitments against reported sum of issued supply and inflation allowance present in the
        // global state match the max supply.
        put     a8[0],ERRNO_INFLATION_MISMATCH;
        // Checking pedersen commitments against reported amount of issued supply present in the
        // global state
        put     a8[1],ERRNO_ISSUED_MISMATCH;
        // initialize index a8[2] to load issue supply in global state
        put     a8[2],0;
        // initialize index a8[3] to load max supply in global state
        put     a8[3],0;
        // initialize offset a16[0] to extract bits from issued supply
        put     a16[0],0;
        // initialize offset a16[1] to extract bits from max supply
        put     a16[1],0;
        // Read issued supply global state from index a8[2] into s16[0]
        ldg     GS_ISSUED_SUPPLY,a8[2],s16[0];
        // Read max supply global state from index a8[3] into s16[1]
        ldg     GS_MAX_SUPPLY,a8[3],s16[1];
        // Extract 64 bits from the beginning of s16[0] into a64[0]
        // NB: if the global state is invalid, we will fail here and fail the validation
        extr    s16[0],a64[0],a16[0];
        // verify sum of pedersen commitments for assignments against a64[0] value
        pcas    OS_ASSET;
        // Extract 64 bits from the beginning of s16[1] from offset a16[0] into a64[1]
        // NB: if the global state is invalid, we will fail here and fail the validation
        extr    s16[1],a64[1],a16[1];
        // verify sum of pedersen commitments for assignments against a64[1] value
        pcas    OS_INFLATION_ALLOWANCE;

        test;
        ret;
    };
    Lib::assemble::<Instr<RgbIsa>>(&code).expect("wrong inflatable asset script")
}

fn ia_schema() -> Schema {
    let types = StandardTypes::with(Rgb20::stl());
    let alu_lib = ia_lib();
    let alu_id = alu_lib.id();
    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("InflatableAsset"),
        timestamp: 1719887551,
        developer: Identity::default(),
        meta_types: tiny_bmap! {
            MS_ALLOWED_INFLATION => types.get("RGBContract.Amount"),
        },
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.ContractTerms")),
            GS_MAX_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_INFLATION_ALLOWANCE => OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
            OS_ASSET => OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_MAX_SUPPLY => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_INFLATION_ALLOWANCE => Occurrences::OnceOrMore,
                OS_ASSET => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            validator: Some(LibSite::with(FN_IA_GENESIS_OFFSET, alu_id)),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_ISSUE => TransitionSchema {
                metadata: none!(),
                globals: tiny_bmap! {
                    GS_ISSUED_SUPPLY => Occurrences::Once,
                },
                inputs: tiny_bmap! {
                    OS_INFLATION_ALLOWANCE => Occurrences::OnceOrMore,
                },
                assignments: tiny_bmap! {
                    OS_INFLATION_ALLOWANCE => Occurrences::OnceOrMore,
                    OS_ASSET => Occurrences::OnceOrMore,
                },
                valencies: none!(),
                validator: None,
            },
            TS_TRANSFER => TransitionSchema {
                metadata: none!(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore,
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore,
                },
                valencies: none!(),
                validator: Some(LibSite::with(FN_IA_TRANSFER_OFFSET, alu_id)),
            }
        },
        reserved: none!(),
    }
}

fn ia_rgb20() -> IfaceImpl {
    let schema = ia_schema();
    let iface = Rgb20::iface(rgb20::Features::INFLATABLE);

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: 1719887551,
        developer: Identity::default(),
        metadata: tiny_bset! {
            NamedField::with(MS_ALLOWED_INFLATION, fname!("allowedInflation")),
        },
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_TERMS, fname!("terms")),
            NamedField::with(GS_MAX_SUPPLY, fname!("maxSupply")),
            NamedField::with(GS_ISSUED_SUPPLY, fname!("issuedSupply"))
        },
        assignments: tiny_bset! {
            NamedField::with(OS_INFLATION_ALLOWANCE, fname!("inflationAllowance")),
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedField::with(TS_ISSUE, fname!("issue")),
            NamedField::with(TS_TRANSFER, fname!("transfer")),
        },
        extensions: none!(),
        errors: tiny_bset! {
            NamedVariant::with(ERRNO_ISSUED_MISMATCH, vname!("issuedMismatch")),
            NamedVariant::with(ERRNO_INFLATION_MISMATCH, vname!("inflationMismatch")),
            NamedVariant::with(ERRNO_NON_EQUAL_IN_OUT, vname!("nonEqualAmounts")),
            NamedVariant::with(ERRNO_INFLATION_EXCEEDED_ALLOWANCE, vname!("inflationExceedsAllowance")),
        },
    }
}

pub struct InflatableAsset;

impl IssuerWrapper for InflatableAsset {
    const FEATURES: rgb20::Features = rgb20::Features::INFLATABLE;
    type IssuingIface = Rgb20;

    fn schema() -> Schema { ia_schema() }

    fn issue_impl() -> IfaceImpl { ia_rgb20() }

    fn types() -> TypeSystem { StandardTypes::with(Rgb20::stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = ia_lib();
        confined_bmap! { lib.id() => lib }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bp::dbc::Method;
    use bp::Txid;
    use rgbstd::interface::*;
    use rgbstd::invoice::Precision;
    use rgbstd::*;

    use super::*;

    #[test]
    fn iimpl_check() {
        let iface = Rgb20::iface(InflatableAsset::FEATURES);
        if let Err(err) = ia_rgb20().check(&iface, &ia_schema()) {
            for e in err {
                eprintln!("{e}");
            }
            panic!("invalid IA RGB20 interface implementation");
        }
    }

    #[test]
    fn test_genesis_inflation_contract() {
        // let created_at = 1713261744;

        let outpoint =
            Txid::from_str("8d54c98d4c29a1ec4fd90635f543f0f7a871a78eb6a6e706342f831d92e3ba19")
                .unwrap();
        let outpoint2 =
            Txid::from_str("d8b91da7d1afc7e2c263413d23239eb6ac4d0f1b7c3c8d83ef2625e9a12cfbad")
                .unwrap();

        let beneficiary = Outpoint::new(outpoint, 1);
        let beneficiary2 = Outpoint::new(outpoint2, 0);
        let contract = Rgb20::testnet::<InflatableAsset>(
            "ssi:anonymous",
            "TEST",
            "Test asset",
            None,
            Precision::CentiMicro,
        )
        .unwrap()
        .allocate(Method::TapretFirst, beneficiary, 0u64)
        .unwrap()
        .allow_inflation(Method::TapretFirst, beneficiary2, 0u64)
        .unwrap()
        .issue_contract()
        .unwrap();

        assert_eq!(
            contract.contract_id().to_string(),
            s!("rgb:qFuT6DN8-9AuO95M-7R8R8Mc-AZvs7zG-obum1Va-BRnweKk")
        );
    }
}
