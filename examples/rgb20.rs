use std::convert::Infallible;
use std::fs;

use amplify::hex::FromHex;
use armor::AsciiArmor;
use bp::dbc::Method;
use bp::{Outpoint, Txid};
use rgb_schemata::NonInflatableAsset;
use rgbstd::containers::FileContent;
use rgbstd::interface::{FilterIncludeAll, FungibleAllocation, IfaceClass, IssuerClass, Rgb20};
use rgbstd::invoice::Precision;
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::resolvers::ResolveHeight;
use rgbstd::validation::{ResolveWitness, WitnessResolverError};
use rgbstd::{WitnessAnchor, WitnessId, XAnchor, XPubWitness};
use strict_encoding::StrictDumb;

struct DumbResolver;

impl ResolveWitness for DumbResolver {
    fn resolve_pub_witness(&self, _: WitnessId) -> Result<XPubWitness, WitnessResolverError> {
        Ok(XPubWitness::strict_dumb())
    }
}

impl ResolveHeight for DumbResolver {
    type Error = Infallible;
    fn resolve_anchor(&mut self, _: &XAnchor) -> Result<WitnessAnchor, Self::Error> {
        Ok(WitnessAnchor::strict_dumb())
    }
}

#[rustfmt::skip]
fn main() {
    let beneficiary_txid = 
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = Outpoint::new(beneficiary_txid, 1);

    let contract = NonInflatableAsset::testnet("TEST", "Test asset", None, Precision::CentiMicro)
        .expect("invalid contract data")
        .allocate(Method::TapretFirst, beneficiary, 1_000_000_000_00u64.into())
        .expect("invalid allocations")
        .issue_contract()
        .expect("invalid contract data");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract.save_file("examples/rgb20-simplest.rgb").expect("unable to save contract");
    fs::write("examples/rgb20-simplest.rgba", contract.to_ascii_armored_string()).expect("unable to save contract");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(Rgb20::iface()).unwrap();
    stock.import_schema(NonInflatableAsset::schema()).unwrap();
    stock.import_iface_impl(NonInflatableAsset::issue_impl()).unwrap();

    stock.import_contract(contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_id(contract_id, Rgb20::iface().iface_id()).unwrap();
    let contract = Rgb20::from(contract);
    let allocations = contract.fungible("assetOwner", &FilterIncludeAll).unwrap();
    eprintln!("\nThe issued contract data:");
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

    for FungibleAllocation  { seal, state, witness, .. } in allocations {
        eprintln!("amount={state}, owner={seal}, witness={witness}");
    }
    eprintln!("totalSupply={}", contract.total_supply());
    eprintln!("created={}", contract.created().to_local().unwrap());
}
