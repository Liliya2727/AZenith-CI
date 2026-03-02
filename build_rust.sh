#!/bin/bash
cd tweakfls
cargo build --release
mkdir -p ../mainfiles/system/bin
cp target/release/sys_azenith_profilesettings ../mainfiles/system/bin/sys.azenith-profilesettings
cp target/release/sys_azenith_utilityconf ../mainfiles/system/bin/sys.azenith-utilityconf
echo "Copied to mainfiles/system/bin"
