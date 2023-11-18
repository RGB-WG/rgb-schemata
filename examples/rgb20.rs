use std::convert::Infallible;
use std::fs;

use amplify::hex::FromHex;
use bp::{Outpoint, Tx, Txid};
use rgb_schemata::{nia_rgb20, nia_schema, OS_ASSET};
use rgbstd::containers::BindleContent;
use rgbstd::interface::{
    rgb20, AssetTagExt, ContractBuilder, FilterIncludeAll, FungibleAllocation, Rgb20,
};
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::resolvers::ResolveHeight;
use rgbstd::stl::{
    Amount, ContractData, DivisibleAssetSpec, Precision, RicardianContract, Timestamp,
};
use rgbstd::validation::{ResolveTx, TxResolverError};
use rgbstd::{Anchor, AssetTag, Layer1, WitnessAnchor};
use strict_encoding::StrictDumb;

struct DumbResolver;

impl ResolveTx for DumbResolver {
    fn resolve_tx(&self, _: Layer1, _: Txid) -> Result<Tx, TxResolverError> {
        Ok(Tx::strict_dumb())
    }
}

impl ResolveHeight for DumbResolver {
    type Error = Infallible;
    fn resolve_anchor(&mut self, _: &Anchor) -> Result<WitnessAnchor, Self::Error> {
        Ok(WitnessAnchor::strict_dumb())
    }
}

#[rustfmt::skip]
fn main() {
    let spec = DivisibleAssetSpec::new("TEST", "Test asset", Precision::CentiMicro);
    let terms = RicardianContract::default();
    let contract_data = ContractData {
        terms,
        media: None,
    };
    let created = Timestamp::now();
    let beneficiary = Outpoint::new(
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap(),
        1
    );

    const ISSUE: u64 = 1_000_000_000_00;
    let asset_tag = AssetTag::new_rgb20("examples.lnp-bp.org", &spec.naming.ticker);

    let contract = ContractBuilder::testnet(
        rgb20(),
        nia_schema(),
        nia_rgb20()
        ).expect("schema fails to implement RGB20 interface")

        .add_global_state("spec", spec)
        .expect("invalid nominal")

        .add_global_state("created", created)
        .expect("invalid creation date")

        .add_global_state("issuedSupply", Amount::from(ISSUE))
        .expect("invalid issued supply")

        .add_global_state("data", contract_data)
        .expect("invalid contract text")

        .add_asset_tag("assetOwner", asset_tag)
        .expect("just one asset tag is used")
        .add_fungible_state("assetOwner", beneficiary, ISSUE)
        .expect("invalid asset amount")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    let bindle = contract.bindle();
    eprintln!("{bindle}");
    bindle.save("examples/rgb20-simplest.rgb").expect("unable to save contract");
    fs::write("examples/rgb20-simplest.rgba", bindle.to_string()).expect("unable to save contract");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(rgb20()).unwrap();
    stock.import_schema(nia_schema()).unwrap();
    stock.import_iface_impl(nia_rgb20()).unwrap();

    // Noe we verify our contract consignment and add it to the stock
    let verified_contract = bindle.unbindle().validate(&mut DumbResolver, true).unwrap_or_else(|consignment| {
        panic!("{}", consignment.validation_status().expect("status always present upon validation"));
    });
    stock.import_contract(verified_contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_id(contract_id, rgb20().iface_id()).unwrap();
    let contract = Rgb20::from(contract);
    let allocations = contract.fungible("assetOwner", &FilterIncludeAll).unwrap();
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

    for FungibleAllocation { owner, witness, value } in allocations {
        eprintln!("amount={value}, owner={owner}, witness={witness}");
    }
    eprintln!("totalSupply={}", contract.total_supply());
    eprintln!("created={}", contract.created().to_local().unwrap());
}
