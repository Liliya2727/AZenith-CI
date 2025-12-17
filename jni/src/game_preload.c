/*
 * Copyright (C) 2024-2025 Zexshia
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <AZenith.h>
#include <dirent.h>
#include <sys/system_properties.h>
int flags = PRELOAD_TOUCH | PRELOAD_VERBOSE;

/***********************************************************************************
 * Function Name      : GamePreload
 * Inputs             : const char* package - target application package name
 * Returns            : void
 * Description        : Preloads all native libraries (.so) inside lib/arm64
 ***********************************************************************************/
void GamePreload(const char* package) {
    sleep(5);

    if (!package || !*package) {
        log_zenith(LOG_WARN, "Package is null or empty");
        return;
    }

    char apk_path[256] = {0};
    char cmd[512];

    snprintf(cmd, sizeof(cmd),
        "cmd package path %s | head -n1 | cut -d: -f2", package);

    FILE* fp = popen(cmd, "r");
    if (!fp || !fgets(apk_path, sizeof(apk_path), fp)) {
        log_zenith(LOG_WARN, "Failed to get APK path");
        if (fp) pclose(fp);
        return;
    }
    pclose(fp);

    apk_path[strcspn(apk_path, "\n")] = 0;
    char* slash = strrchr(apk_path, '/');
    if (!slash) return;
    *slash = '\0';

    char lib_path[300];
    snprintf(lib_path, sizeof(lib_path), "%s/lib/arm64", apk_path);

    char budget_prop[32] = {0};
    __system_property_get(
        "persist.sys.azenithconf.preloadbudget",
        budget_prop
    );

    size_t max_bytes = 0;
    if (*budget_prop)
        max_bytes = parse_size(budget_prop); // reuse your util

    preload_stats_t stats = {0};
    
    log_zenith(LOG_INFO, "Native preload start for %s", package);
    
    if (access(lib_path, F_OK) == 0) {
        preload_path_native(lib_path, max_bytes, flags, &stats);
    } else {
        preload_path_native(apk_path, max_bytes, flags, &stats);
    }
    
    log_preload(
        LOG_INFO,
        "Game %s preloaded: %lld pages (~%lld MB)",
        package,
        (long long)stats.pages_touched,
        (long long)(stats.bytes_touched >> 20)
    );
}
