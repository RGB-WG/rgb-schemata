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

use std::io::stdout;
use std::{fs, io};

use armor::AsciiArmor;
use rgb_schemata::{cfa_rgb25, cfa_schema, uda_rgb21, uda_schema, NonInflatableAsset};
use rgbstd::containers::FileContent;
use rgbstd::interface::{rgb21, rgb25, IfaceClass, IssuerClass, Rgb20};
use rgbstd::vm::RgbIsa;
use rgbstd::SubSchema;

fn main() -> io::Result<()> {
    let rgb20 = Rgb20::iface();
    rgb20.save_file("interfaces/RGB20.rgb")?;
    fs::write("interfaces/RGB20.rgba", rgb20.to_ascii_armored_string())?;

    let rgb21 = rgb21();
    rgb21.save_file("interfaces/RGB21.rgb")?;
    fs::write("interfaces/RGB21.rgba", rgb21.to_ascii_armored_string())?;

    let rgb25 = rgb25();
    rgb25.save_file("interfaces/RGB25.rgb")?;
    fs::write("interfaces/RGB25.rgba", rgb25.to_ascii_armored_string())?;

    nia()?;
    uda()?;
    cfa()?;

    Ok(())
}

fn nia() -> io::Result<()> {
    let schema = NonInflatableAsset::schema();
    schema.save_file("schemata/NonInflatableAssets.rgb")?;
    fs::write("schemata/NonInflatableAssets.rgba", schema.to_ascii_armored_string())?;
    print_lib(&schema);

    let iimpl = NonInflatableAsset::issue_impl();
    iimpl.save_file("schemata/NonInflatableAssets-RGB20.rgb")?;
    fs::write("schemata/NonInflatableAssets-RGB20.rgba", iimpl.to_ascii_armored_string())?;

    Ok(())
}

fn uda() -> io::Result<()> {
    let schema = uda_schema();
    schema.save_file("schemata/UniqueDigitalAsset.rgb")?;
    fs::write("schemata/UniqueDigitalAsset.rgba", schema.to_ascii_armored_string())?;
    print_lib(&schema);

    let iimpl = uda_rgb21();
    iimpl.save_file("schemata/UniqueDigitalAsset-RGB21.rgb")?;
    fs::write("schemata/UniqueDigitalAsset-RGB21.rgba", iimpl.to_ascii_armored_string())?;

    Ok(())
}

fn cfa() -> io::Result<()> {
    let schema = cfa_schema();
    schema.save_file("schemata/CollectibleFungibleAssets.rgb")?;
    fs::write("schemata/CollectibleFungibleAssets.rgba", schema.to_ascii_armored_string())?;
    print_lib(&schema);

    let iimpl = cfa_rgb25();
    iimpl.save_file("schemata/CollectibleFungibleAssets-RGB25.rgb")?;
    fs::write("schemata/CollectibleFungibleAssets-RGB25.rgba", iimpl.to_ascii_armored_string())?;

    Ok(())
}

fn print_lib(schema: &SubSchema) {
    let alu_lib = schema.script.as_alu_script().libs.values().next().unwrap();
    eprintln!("{alu_lib}");
    alu_lib.print_disassemble::<RgbIsa>(stdout()).ok();
}
