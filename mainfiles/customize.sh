#
# Copyright (C) 2024-2025 Zexshia
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

SKIPUNZIP=1

# Paths
MODULE_CONFIG="/sdcard/AZenith/config"
VALUE_DIR="$MODULE_CONFIG/value"   
device_codename=$(getprop ro.product.board)
chip=$(getprop ro.hardware)

# Create File
make_node() {
	[ ! -f "$2" ] && echo "$1" >"$2"
}

# Displaybanner
ui_print ""
ui_print "              AetherZenith              "
ui_print ""
ui_print "- Installing AetherZenith..."

# Extract Module Directiories
mkdir -p "$MODULE_CONFIG"
mkdir -p "$MODULE_CONFIG/debug"
mkdir -p "$MODULE_CONFIG/API"
mkdir -p "$MODULE_CONFIG/preload"
mkdir -p "$MODULE_CONFIG/gamelist"
ui_print "- Create module config"

# Flashable integrity checkup
ui_print "- Extracting verify.sh"
unzip -o "$ZIPFILE" 'verify.sh' -d "$TMPDIR" >&2
[ ! -f "$TMPDIR/verify.sh" ] && abort_corrupted
source "$TMPDIR/verify.sh"

# Extract Module files
ui_print "- Extracting system directory..."
extract "$ZIPFILE" 'system/bin/sys.aetherzenith-profiles' "$MODPATH"
extract "$ZIPFILE" 'system/bin/sys.aetherzenith-conf' "$MODPATH"
extract "$ZIPFILE" 'system/bin/sys.aetherzenith-preload' "$MODPATH"
extract "$ZIPFILE" 'system/bin/sys.aetherzenith-thermalcore' "$MODPATH"
ui_print "- Extracting service.sh..."
extract "$ZIPFILE" service.sh "$MODPATH"
ui_print "- Extracting module.prop..."
extract "$ZIPFILE" module.prop "$MODPATH"
ui_print "- Extracting uninstall.sh..."
extract "$ZIPFILE" uninstall.sh "$MODPATH"
ui_print "- Extracting gamelist.txt..."
extract "$ZIPFILE" gamelist.txt "$MODULE_CONFIG/gamelist"
ui_print "- Extracting module icon..."
extract "$ZIPFILE" module.avatar.webp "/data/local/tmp"
ui_print "- Extracting module banner..."
extract "$ZIPFILE" module.banner.avif "$MODPATH"

# Target architecture
case $ARCH in
"arm64") ARCH_TMP="arm64-v8a" ;;
"arm") ARCH_TMP="armeabi-v7a" ;;
"x64") ARCH_TMP="x86_64" ;;
"x86") ARCH_TMP="x86" ;;
"riscv64") ARCH_TMP="riscv64" ;;
*) abort ;;
esac

# Extract daemon
ui_print "- Extracting sys.aetherzenith-service for $ARCH_TMP"
extract "$ZIPFILE" "libs/$ARCH_TMP/sys-aetherzenith-service" "$TMPDIR"
cp "$TMPDIR"/libs/"$ARCH_TMP"/* "$MODPATH/system/bin"
ln -sf "$MODPATH/system/bin/sys.aetherzenith-service" "$MODPATH/system/bin/sys.etherzenith-log"
ln -sf "$MODPATH/system/bin/sys.aetherzenith-service" "$MODPATH/system/bin/aetherzenith"
rm -rf "$TMPDIR/libs"
ui_print "- Installing for Arch : $ARCH_TMP"

# Apply Tweaks Based on Chipset
ui_print "- Checking device soc"
chipset=$(grep -i 'hardware' /proc/cpuinfo | uniq | cut -d ':' -f2 | sed 's/^[ \t]*//')
[ -z "$chipset" ] && chipset="$(getprop ro.board.platform) $(getprop ro.hardware)"

case "$(echo "$chipset" | tr '[:upper:]' '[:lower:]')" in
*mt* | *MT*)
	soc="MediaTek"
	ui_print "- Applying Tweaks for $soc"
	echo 1 > $VALUEDIR/soctype
	;;
*sm* | *qcom* | *SM* | *QCOM* | *Qualcomm* | *sdm* | *snapdragon*)
	soc="Snapdragon"
	ui_print "- Applying Tweaks for $soc"
	echo 2 > $VALUEDIR/soctype
	;;
*exynos* | *Exynos* | *EXYNOS* | *universal* | *samsung* | *erd* | *s5e*)
	soc="Exynos"
	ui_print "- Applying Tweaks for $soc"
	echo 3 > $VALUEDIR/soctype
	;;
*Unisoc* | *unisoc* | *ums*)
	soc="Unisoc"
	ui_print "- Applying Tweaks for $soc"
	echo 4 > $VALUEDIR/soctype
	;;
*gs* | *Tensor* | *tensor*)
	soc="Tensor"
	ui_print "- Applying Tweaks for $soc"
	echo 5 > $VALUEDIR/soctype
	;;
*)
	soc="Unknown"
	ui_print "- Applying Tweaks for $chipset"
	echo 0 > $VALUEDIR/soctype
	;;
esac

# Soc Type
# 1) MediaTek
# 2) Snapdragon
# 3) Exynos
# 4) Unisoc
# 5) Tensor
# 0) Unknown

# Extract Webui
ui_print "- Extracting WebUI"
mkdir -p "$MODPATH/webroot"
unzip -o "$ZIPFILE" "webroot/*" -d "$TMPDIR" >&2
cp -r "$TMPDIR/webroot/"* "$MODPATH/webroot/"
rm -rf "$TMPDIR/webroot"


# Set AZenith Config Files
mkdir -p "$VALUE_DIR"
ui_print "- Setting up AZenith Config Files"
echo "Disabled 90% 80% 70% 60% 50% 40%" >"$VALUE_DIR/freqlist"
echo "Disabled 60hz 90hz 120hz" >"$VALUE_DIR/vsynclist"

# Default values for specific configs
[ ! -f "$VALUE_DIR/freqoffset" ] && echo "Disabled" >"$VALUE_DIR/freqoffset"
[ ! -f "$VALUE_DIR/vsync" ] && echo "Disabled" >"$VALUE_DIR/vsync"
[ ! -f "$VALUE_DIR/schemeconfig" ] && echo "1000 1000 1000 1000" >"$VALUE_DIR/schemeconfig"

ui_print "- Creating default config files..."

props="
logd
DThermal
SFL
malisched
fpsged
schedtunes
clearbg
bypasschg
APreload
iosched
cpulimit
dnd
justintime
disabletrace
thermalcore
showtoast
AIenabled
debugmode
"

# Generate files for all properties if missing
for name in $props; do
  path="$VALUE_DIR/$name"
  case "$name" in
    showtoast)
      [ ! -f "$path" ] && echo "1" >"$path"
      ;;
    AIenabled)
      [ ! -f "$path" ] && echo "1" >"$path"
      ;;
    debugmode)
      [ ! -f "$path" ] && echo "false" >"$path"
      ;;
    *)
      [ ! -f "$path" ] && echo "0" >"$path"
      ;;
  esac
done

ui_print "- All AZenith config values saved in:"
ui_print "  $VALUE_DIR/"

# Install toast if not installed
if pm list packages | grep -q azenith.toast; then
	ui_print "- AZenith Toast is already installed."
else
	ui_print "- Extracting AZenith Toast..."
	unzip -qo "$ZIPFILE" azenithtoast.apk -d "$MODPATH" >&2
	ui_print "- Installing AZenith Toast..."
	pm install "$MODPATH/azenithtoast.apk"
	rm "$MODPATH/azenithtoast.apk"
fi

# Set Permissions
ui_print "- Setting Permissions..."
set_perm_recursive "$MODPATH/system/bin" 0 2000 0777 0777
chmod +x "$MODPATH/system/bin/sys.aetherzenith-service"
chmod +x "$MODPATH/system/bin/sys.aetherzenith-profiles"
chmod +x "$MODPATH/system/bin/sys.aetherzenith-conf"
chmod +x "$MODPATH/system/bin/sys.aetherzenith-preload"
chmod +x "$MODPATH/system/bin/sys.aetherzenith-thermalcore"

ui_print "- Installation complete!"
