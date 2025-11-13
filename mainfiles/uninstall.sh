#!/bin/sh

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


# Uninstall module directories
rm -rf /sdcard/AZenith
rm -rf /data/AZenith
# Uninstall toast apk
pm uninstall azenith.toast 2>/dev/null
# Uninstaller Script
manager_paths="$MODPATH/system/bin"
binaries="sys.aetherzenith-service sys.aetherzenith-log \
          sys.aetherzenith-profiles sys.aetherzenith-conf \
          sys.aetherzenith-preload sys.aetherzenith-thermalservice"
for dir in $manager_paths; do
	[ -d "$dir" ] || continue
	for remove in $binaries; do
		link="$dir/$remove"
		rm -f "$link"
	done
done
