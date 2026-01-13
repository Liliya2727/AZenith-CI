package zx.azenith.ui.viewmodel

import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import com.topjohnwu.superuser.io.SuFile
import com.topjohnwu.superuser.io.SuFileInputStream   
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import kotlinx.serialization.json.Json
import zx.azenith.ui.util.AppConfig

data class AppInfo(
    val label: String,
    val packageName: String,
    val uid: Int,
    val isSystem: Boolean,
    val packageInfo: android.content.pm.PackageInfo, // Pastikan ini ada
    val isRecommended: Boolean = false,
    val isEnabledInConfig: Boolean = false
)



class ApplistViewmodel : ViewModel() {
    // State disimpan di level ViewModel agar tidak hilang saat pindah tab
    var rawApps by mutableStateOf<List<AppInfo>>(emptyList())
        private set

    var isRefreshing by mutableStateOf(false)
        private set

    private val configPath = "/data/adb/.config/AZenith/gamelist/azenithApplist.json"
    private val jsonHandler = Json { 
        prettyPrint = true
        ignoreUnknownKeys = true 
        encodeDefaults = true 
    }

    // PINDAHKAN KE SINI
    fun refreshAppConfigStatus() {
        viewModelScope.launch(Dispatchers.IO) {
            val file = SuFile(configPath)
            if (file.exists()) {
                val content = try {
                    SuFileInputStream.open(file).bufferedReader().use { it.readText() }
                } catch (e: Exception) { "" }

                if (content.isNotEmpty()) {
                    try {
                        val decoded = jsonHandler.decodeFromString<Map<String, AppConfig>>(content)
                        val enabledPackages = decoded.keys
                        
                        withContext(Dispatchers.Main) {
                            rawApps = rawApps.map { app ->
                                app.copy(isEnabledInConfig = enabledPackages.contains(app.packageName))
                            }
                        }
                    } catch (e: Exception) { e.printStackTrace() }
                }
            } else {
                withContext(Dispatchers.Main) {
                    rawApps = rawApps.map { it.copy(isEnabledInConfig = false) }
                }
            }
        }
    }
    
    fun loadApps(context: Context, forceRefresh: Boolean = false, onComplete: () -> Unit = {}) {
        // Logika agar tidak load berulang kali
        if (!forceRefresh && rawApps.isNotEmpty()) {
            onComplete()
            return
        }

        viewModelScope.launch(Dispatchers.IO) {
            isRefreshing = true
            withContext(Dispatchers.Main) { isRefreshing = true }
            val pm = context.packageManager
            
            // 1. Ambil list rekomendasi dari Assets
            val gameList = try {
                context.assets.open("gamelist.txt").bufferedReader().useLines { it.toSet() }
            } catch (e: Exception) { emptySet() }

            // 2. Ambil list yang sudah di-enable dari JSON config
            val enabledList = getEnabledPackages()

            // 3. Ambil data aplikasi dari sistem
            val installed = pm.getInstalledPackages(PackageManager.GET_META_DATA)
            // Di dalam loadApps loop:
            // Di dalam loop loadApps ViewModel
            val loadedApps = installed.map { pkg ->
                AppInfo(
                    label = pkg.applicationInfo?.loadLabel(pm)?.toString() ?: "Unknown",
                    packageName = pkg.packageName,
                    uid = pkg.applicationInfo?.uid ?: 0,
                    isSystem = pkg.applicationInfo?.let { (it.flags and ApplicationInfo.FLAG_SYSTEM) != 0 } ?: false,
                    packageInfo = pkg, // Simpan objek PackageInfo di sini
                    isRecommended = gameList.contains(pkg.packageName),
                    isEnabledInConfig = enabledList.contains(pkg.packageName)
                )
            }.sortedWith(
                // 1. Urutkan berdasarkan status Enabled (Boolean true di Kotlin dianggap lebih besar dari false)
                // Kita gunakan compareByDescending agar true (Enabled) muncul di atas.
                compareByDescending<AppInfo> { it.isEnabledInConfig }
                    // 2. Jika statusnya sama (sama-sama enabled atau sama-sama disabled), urutkan A-Z
                    .thenBy { it.label.lowercase() }
            )
                        
            withContext(Dispatchers.Main) {
                rawApps = loadedApps
                isRefreshing = false
                onComplete()
            }
        }
    }

    private fun getEnabledPackages(): Set<String> {
        val set = mutableSetOf<String>()
        try {
            val path = "/data/adb/.config/AZenith/gamelist/azenithApplist.json"
            val suFile = SuFile(path)
    
            if (suFile.exists()) {
                // Membaca file menggunakan Stream khusus Root dari libsu
                val content = SuFileInputStream.open(suFile).bufferedReader().use { it.readText() }
                
                if (content.isNotBlank()) {
                    val json = JSONObject(content)
                    val keys = json.keys()
                    while (keys.hasNext()) {
                        set.add(keys.next())
                    }
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
        return set
    }
}
