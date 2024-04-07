use std::fs;

use amplify::hex::FromHex;
use bp::Txid;
use rgb_schemata::dumb::DumbResolver;
use rgb_schemata::{cfa_rgb25, cfa_schema};
use rgbstd::containers::FileContent;
use rgbstd::interface::{
    rgb25, ContractBuilder, FilterIncludeAll, FungibleAllocation, IfaceClass, IfaceWrapper, Rgb25,
};
use rgbstd::invoice::{Amount, Precision};
use rgbstd::persistence::{Inventory, Stock};
use rgbstd::stl::{AssetTerms, Attachment, Details, MediaType, Name, RicardianContract};
use rgbstd::{GenesisSeal, XChain};
use sha2::{Digest, Sha256};

#[rustfmt::skip]
fn main() {
    let name = Name::try_from("Test asset").unwrap();
    let details = Details::try_from("details with ℧nicode characters").unwrap();
    let precision = Precision::CentiMicro;

    let file_bytes = std::fs::read("README.md").unwrap();
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    let file_hash = hasher.finalize();
    let terms = AssetTerms {
        text: RicardianContract::default(),
        media: Some(Attachment {
            ty: MediaType::with("text/*"),
            digest: file_hash.into(),
        }),
    };
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = XChain::Bitcoin(GenesisSeal::tapret_first_rand(beneficiary_txid, 1));

    const ISSUE: u64 = 1_000_000_000_000;

    let contract = ContractBuilder::testnet(
        Rgb25::iface(rgb25::Features::none()),
        cfa_schema(),
        cfa_rgb25()
        ).expect("schema fails to implement RGB25 interface")

        .add_global_state("name", name)
        .expect("invalid name")

        .add_global_state("details", details)
        .expect("invalid details")

        .add_global_state("precision", precision)
        .expect("invalid precision")

        .add_global_state("issuedSupply", Amount::from(ISSUE))
        .expect("invalid issuedSupply")

        .add_global_state("terms", terms)
        .expect("invalid contract data")

        .add_fungible_state("assetOwner", beneficiary, ISSUE)
        .expect("invalid asset amount")

        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();
    debug_assert_eq!(contract_id, contract.contract_id());

    eprintln!("{contract}");
    contract.save_file("examples/rgb25-simplest.rgb").expect("unable to save contract");
    fs::write("examples/rgb25-simplest.rgba", contract.to_string()).expect("unable to save contract");

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::default();
    stock.import_iface(Rgb25::iface(rgb25::Features::all())).unwrap();
    stock.import_iface(Rgb25::iface(rgb25::Features::none())).unwrap();
    stock.import_schema(cfa_schema()).unwrap();
    stock.import_iface_impl(cfa_rgb25()).unwrap();

    // Noe we verify our contract consignment and add it to the stock
    let verified_contract = match contract.validate(&mut DumbResolver, true) {
        Ok(consignment) => consignment,
        Err(consignment) => {
            panic!("can't produce valid consignment. Report: {}", consignment.validation_status().expect("status always present upon validation"));
        }
    };
    stock.import_contract(verified_contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_id(contract_id, Rgb25::IFACE_ID).unwrap();
    let name = contract.global("name").unwrap();
    let allocations = contract.fungible("assetOwner", &FilterIncludeAll).unwrap();
    eprintln!("{}", Name::from_strict_val_unchecked(&name[0]));

    for FungibleAllocation { seal, state, witness, .. } in allocations {
        eprintln!("(amount={state}, owner={seal}, witness={witness})");
    }
}
