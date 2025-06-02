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

use std::io;
use std::io::stdout;

use rgbstd::containers::{FileContent, Kit};
use rgbstd::contract::IssuerWrapper;
use rgbstd::persistence::MemContract;
use rgbstd::vm::RgbIsa;
use schemata::{
    CollectibleFungibleAsset, InflatableFungibleAsset, NonInflatableAsset,
    PermissionedFungibleAsset, UniqueDigitalAsset,
};

fn main() -> io::Result<()> {
    cfa()?;
    ifa()?;
    nia()?;
    pfa()?;
    uda()?;

    Ok(())
}

fn nia() -> io::Result<()> {
    let schema = NonInflatableAsset::schema();
    let lib = NonInflatableAsset::scripts();
    let types = NonInflatableAsset::types();

    let mut kit = Kit::default();
    kit.schemata.push(schema).unwrap();
    kit.scripts.extend(lib.into_values()).unwrap();
    kit.types = types;

    kit.save_file("schemata/NonInflatableAsset.rgb")?;
    kit.save_armored("schemata/NonInflatableAsset.rgba")?;
    print_lib(&kit);

    Ok(())
}

fn pfa() -> io::Result<()> {
    let schema = PermissionedFungibleAsset::schema();
    let lib = PermissionedFungibleAsset::scripts();
    let types = PermissionedFungibleAsset::types();

    let mut kit = Kit::default();
    kit.schemata.push(schema).unwrap();
    kit.scripts.extend(lib.into_values()).unwrap();
    kit.types = types;

    kit.save_file("schemata/PermissionedFungibleAsset.rgb")?;
    kit.save_armored("schemata/PermissionedFungibleAsset.rgba")?;
    print_lib(&kit);

    Ok(())
}

fn uda() -> io::Result<()> {
    let schema = UniqueDigitalAsset::schema();
    let lib = UniqueDigitalAsset::scripts();
    let types = UniqueDigitalAsset::types();

    let mut kit = Kit::default();
    kit.schemata.push(schema).unwrap();
    kit.scripts.extend(lib.into_values()).unwrap();
    kit.types = types;

    kit.save_file("schemata/UniqueDigitalAsset.rgb")?;
    kit.save_armored("schemata/UniqueDigitalAsset.rgba")?;
    print_lib(&kit);

    Ok(())
}

fn cfa() -> io::Result<()> {
    let schema = CollectibleFungibleAsset::schema();
    let lib = CollectibleFungibleAsset::scripts();
    let types = CollectibleFungibleAsset::types();

    let mut kit = Kit::default();
    kit.schemata.push(schema).unwrap();
    kit.scripts.extend(lib.into_values()).unwrap();
    kit.types = types;

    kit.save_file("schemata/CollectibleFungibleAsset.rgb")?;
    kit.save_armored("schemata/CollectibleFungibleAsset.rgba")?;
    print_lib(&kit);

    Ok(())
}

fn ifa() -> io::Result<()> {
    let schema = InflatableFungibleAsset::schema();
    let lib = InflatableFungibleAsset::scripts();
    let types = InflatableFungibleAsset::types();

    let mut kit = Kit::default();
    kit.schemata.push(schema).unwrap();
    kit.scripts.extend(lib.into_values()).unwrap();
    kit.types = types;

    kit.save_file("schemata/InflatableFungibleAsset.rgb")?;
    kit.save_armored("schemata/InflatableFungibleAsset.rgba")?;
    print_lib(&kit);

    Ok(())
}

fn print_lib(kit: &Kit) {
    let alu_lib = kit.scripts.first().unwrap();
    eprintln!("{alu_lib}");
    alu_lib
        .print_disassemble::<RgbIsa<MemContract>>(stdout())
        .unwrap();
}
