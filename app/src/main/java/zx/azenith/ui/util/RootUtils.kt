package zx.azenith.ui.util

import com.topjohnwu.superuser.Shell
import com.topjohnwu.superuser.io.SuFile

object RootUtils {

    // 1. Cek Root Access
    fun isRootGranted(): Boolean {
        return Shell.getShell().isRoot
    }

    // 2. Ambil Current Profile dari File
    fun getCurrentProfile(): String {
        val path = "/data/adb/.config/AZenith/API/current_profile"
        val file = SuFile(path)
        
        if (!file.exists() || !file.canRead()) return "Unknown"

        return try {
            val content = file.newInputStream().bufferedReader().use { it.readLine()?.trim() }
            when (content) {
                "0" -> "Initializing"
                "1" -> "Performance"
                "2" -> "Balanced"
                "3" -> "ECO Mode"
                else -> "Unknown"
            }
        } catch (e: Exception) {
            "Unknown"
        }
    }

    // 3. Ambil Service PID (pidof)
    // Return Pair(Status, PID)
    fun getServiceStatus(): Pair<String, String> {
        val result = Shell.cmd("pidof sys.azenith-service").exec()
        return if (result.isSuccess) {
            val pid = result.out.firstOrNull() ?: ""
            "Alive" to pid
        } else {
            "Suspended" to ""
        }
    }
}
