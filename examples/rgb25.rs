use std::convert::Infallible;
use std::fs;

use amplify::hex::FromHex;
use bp::{Outpoint, Tx, Txid};
use rgb_schemata::{cfa_rgb25, cfa_schema};
use rgbstd::containers::BindleContent;
use rgbstd::interface::{rgb25, ContractBuilder, FilterIncludeAll, FungibleAllocation};
use rgbstd::invoice::{Amount, Precision};
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::resolvers::ResolveHeight;
use rgbstd::stl::{
    Attachment, ContractData, Details, MediaType, Name, RicardianContract, Timestamp,
};
use rgbstd::validation::{ResolveTx, TxResolverError};
use rgbstd::{Anchor, Layer1, WitnessAnchor};
use sha2::{Digest, Sha256};
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
    let name = Name::try_from("Test asset").unwrap();
    let details = Details::try_from("details with ℧nicode characters").unwrap();
    let precision = Precision::CentiMicro;
    let terms = RicardianContract::default();
    let file_bytes = std::fs::read("README.md").unwrap();
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    let file_hash = hasher.finalize();
    let media = Some(Attachment {
        ty: MediaType::with("text/*"),
        digest: file_hash.into(),
    });
    let contract_data = ContractData {
        terms,
        media,
    };
    let created = Timestamp::now();
    let beneficiary = Outpoint::new(
        Txid::from_hex("623554ac1dcd15496c105a27042c438921f2a82873579be88e74d7ef559a3d91").unwrap(),
        0
    );

    const ISSUE: u64 = 1_000_000_000_000;

    let contract = ContractBuilder::testnet(
        rgb25(),
        cfa_schema(),
        cfa_rgb25()
        ).expect("schema fails to implement RGB25 interface")

        .add_global_state("name", name)
        .expect("invalid name")

        .add_global_state("details", details)
        .expect("invalid details")

        .add_global_state("precision", precision)
        .expect("invalid precision")

        .add_global_state("created", created)
        .expect("invalid created")

        .add_global_state("issuedSupply", Amount::from(ISSUE))
        .expect("invalid issuedSupply")

        .add_global_state("data", contract_data)
        .expect("invalid contract data")

        .add_fungible_state("assetOwner", beneficiary, ISSUE)
        .expect("invalid asset amount")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();
    debug_assert_eq!(contract_id, contract.contract_id());

    let bindle = contract.bindle();
    eprintln!("{bindle}");
    bindle.save("examples/rgb25-simplest.contract.rgb").expect("unable to save contract");
    fs::write("examples/rgb25-simplest.contract.rgba", bindle.to_string()).expect("unable to save contract");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(rgb25()).unwrap();
    stock.import_schema(cfa_schema()).unwrap();
    stock.import_iface_impl(cfa_rgb25()).unwrap();

    // Noe we verify our contract consignment and add it to the stock
    let verified_contract = match bindle.unbindle().validate(&mut DumbResolver, true) {
        Ok(consignment) => consignment,
        Err(consignment) => {
            panic!("can't produce valid consignment. Report: {}", consignment.validation_status().expect("status always present upon validation"));
        }
    };
    stock.import_contract(verified_contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_id(contract_id, rgb25().iface_id()).unwrap();
    let name = contract.global("name").unwrap();
    let allocations = contract.fungible("assetOwner", &FilterIncludeAll).unwrap();
    eprintln!("{}", Name::from_strict_val_unchecked(&name[0]));

    for FungibleAllocation { owner, witness, value } in allocations {
        eprintln!("(amount={value}, owner={owner}, witness={witness})");
    }
}
