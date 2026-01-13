package zx.azenith.receiver

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.widget.Toast
import androidx.core.app.NotificationCompat
import zx.azenith.R
import java.util.Locale

class ZenithReceiver : BroadcastReceiver() {

    companion object {
        private const val CH_PROFILE = "az_profile"
        private const val CH_SYSTEM = "az_system"
        private const val PROFILE_ID = 1001
        const val ACTION_TOAST = "zx.azenith.ACTION_TOAST"
        const val ACTION_NOTIFY = "zx.azenith.ACTION_NOTIFY"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action ?: return
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        when (action) {
            ACTION_TOAST -> {
                val msg = intent.getStringExtra("message")
                if (!msg.isNullOrEmpty()) {
                    Toast.makeText(context, msg, Toast.LENGTH_LONG).show()
                }
            }
            ACTION_NOTIFY -> {
                handleNotification(context, intent, manager)
            }
        }
    }

    private fun handleNotification(context: Context, intent: Intent, manager: NotificationManager) {
        val title = intent.getStringExtra("title") ?: "AZenith"
        val text = intent.getStringExtra("text") ?: return
        val chrono = intent.getBooleanExtra("chrono", false)
        val timeout = intent.getLongExtra("timeout", 0L)

        val isProfile = title.lowercase(Locale.ROOT).contains("profile") || 
                        title.lowercase(Locale.ROOT).contains("mode")
        
        val channelId = if (isProfile) CH_PROFILE else CH_SYSTEM

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channelName = if (isProfile) "AZenith Profiles" else "AZenith System"
            val channel = NotificationChannel(channelId, channelName, NotificationManager.IMPORTANCE_LOW)
            manager.createNotificationChannel(channel)
        }

        val builder = NotificationCompat.Builder(context, channelId)
            .setSmallIcon(R.mipmap.ic_launcher) // Gunakan R.mipmap.ic_launcher agar lebih aman
            .setContentTitle(title)
            .setContentText(text)
            .setOngoing(true)
            .setUsesChronometer(chrono)
            .setAutoCancel(true)

        if (timeout > 0L) {
            builder.setTimeoutAfter(timeout)
        }

        val notificationId = if (isProfile) PROFILE_ID else title.hashCode()
        manager.notify(notificationId, builder.build())
    }
}
