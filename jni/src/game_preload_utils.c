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

#include <dirent.h>
#include <fcntl.h>
#include <limits.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>

#include <AZenith.h>

long g_pagesize = 0;

/***********************************************************************************
 * Function Name      : is_preload_target
 * Inputs             : const char* name - file path or name
 * Returns            : int (1 if preloadable, 0 otherwise)
 * Description        : Checks whether a file is a valid preload target
 ***********************************************************************************/
int is_preload_target(const char* name) {
    static const char* exts[] = {
        ".so", ".apk", ".odex", ".vdex", ".art", ".dm"
    };

    for (size_t i = 0; i < sizeof(exts) / sizeof(exts[0]); i++) {
        if (strstr(name, exts[i]))
            return 1;
    }
    return 0;
}

/***********************************************************************************
 * Function Name      : preload_file
 * Inputs             : const char* path       - target file path
 *                      size_t max_bytes       - maximum bytes to preload (0 = no limit)
 *                      preload_stats_t* stats - preload statistics output
 * Returns            : int (0 on success, -1 on failure)
 * Description        : Memory-maps and touches pages of a single file
 ***********************************************************************************/
int preload_file(
    const char* path,
    size_t max_bytes,
    preload_stats_t* stats
) {
    if (!path || !stats)
        return -1;

    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0)
        return -1;

    struct stat st;
    if (fstat(fd, &st) < 0 || st.st_size <= 0) {
        close(fd);
        return -1;
    }

    size_t size = st.st_size;
    if (max_bytes && size > max_bytes)
        size = max_bytes;

    void* mem = mmap(NULL, size, PROT_READ, MAP_SHARED, fd, 0);
    if (mem == MAP_FAILED) {
        close(fd);
        return -1;
    }

    if (!g_pagesize)
        g_pagesize = sysconf(_SC_PAGESIZE);

    volatile unsigned char* p = (volatile unsigned char*)mem;
    size_t pages = (size + g_pagesize - 1) / g_pagesize;

    for (size_t i = 0; i < pages; i++) {
        (void)p[i * g_pagesize];   // VM page fault
    }

    munmap(mem, size);
    close(fd);

    stats->pages_touched += pages;
    stats->bytes_touched += size;

    log_preload(LOG_DEBUG, "Touched %s (%zu pages)", path, pages);
    return 0;
}

/***********************************************************************************
 * Function Name      : preload_crawl
 * Inputs             : const char* path       - directory or file path
 *                      size_t max_bytes       - preload byte limit
 *                      preload_stats_t* stats - preload statistics output
 * Returns            : void
 * Description        : Recursively scans directories and preloads valid files
 ***********************************************************************************/
void preload_crawl(
    const char* path,
    size_t max_bytes,
    preload_stats_t* stats
) {
    struct stat st;
    if (lstat(path, &st) < 0)
        return;

    if (S_ISDIR(st.st_mode)) {
        DIR* d = opendir(path);
        if (!d)
            return;

        struct dirent* e;
        char buf[PATH_MAX];

        while ((e = readdir(d))) {
            if (!strcmp(e->d_name, ".") || !strcmp(e->d_name, ".."))
                continue;

            snprintf(buf, sizeof(buf), "%s/%s", path, e->d_name);
            preload_crawl(buf, max_bytes, stats);

            if (max_bytes && stats->bytes_touched >= max_bytes)
                break;
        }
        closedir(d);
    }
    else if (S_ISREG(st.st_mode)) {
        if (is_preload_target(path)) {
            preload_file(path, max_bytes, stats);
        }
    }
}

/***********************************************************************************
 * Function Name      : preload_path_native
 * Inputs             : const char* path       - root path to preload
 *                      size_t max_bytes       - maximum bytes to preload
 *                      preload_stats_t* stats - preload statistics output
 * Returns            : int (0 on success, -1 on failure)
 * Description        : Entry point for native VM-based preloading
 ***********************************************************************************/
int preload_path_native(
    const char* path,
    size_t max_bytes,
    preload_stats_t* stats
) {
    if (!path || !stats)
        return -1;

    preload_crawl(path, max_bytes, stats);
    return 0;
}

/***********************************************************************************
 * Function Name      : parse_size
 * Inputs             : const char* str - size string (e.g. "512M", "1G", "1024K")
 * Returns            : size_t - size in bytes
 * Description        : Parses human-readable memory size strings
 ***********************************************************************************/
size_t parse_size(const char* str) {
    if (!str || !*str)
        return 0;

    char* end;
    unsigned long long value = strtoull(str, &end, 10);

    if (end == str)
        return 0;

    switch (*end) {
        case 'G':
        case 'g':
            value <<= 30;
            break;
        case 'M':
        case 'm':
            value <<= 20;
            break;
        case 'K':
        case 'k':
            value <<= 10;
            break;
        default:
            break; // bytes
    }

    return (size_t)value;
}
