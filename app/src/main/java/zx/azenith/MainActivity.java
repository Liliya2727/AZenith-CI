package zx.azenith;

import android.app.Activity;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.widget.Toast;

public class MainActivity extends Activity {
    // Bedakan Channel agar tidak saling ganggu
    private static final String CH_PROFILE = "az_profile";
    private static final String CH_SYSTEM = "az_system";
    private static final int PROFILE_ID = 1001;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
    
        // Ambil data intent
        String clearAll = getIntent().getStringExtra("clearall");
        String toastMsg = getIntent().getStringExtra("toasttext");
        String notifyTitle = getIntent().getStringExtra("notifytitle");
        String notifyMsg = getIntent().getStringExtra("notifytext");
        String useChrono = getIntent().getStringExtra("chrono");
        String timeoutStr = getIntent().getStringExtra("timeout");
    
        NotificationManager manager = (NotificationManager) getSystemService(NOTIFICATION_SERVICE);
    
        if ("true".equals(clearAll)) {
            manager.cancelAll();
        }
    
        if (toastMsg != null) {
            Toast.makeText(this, toastMsg, Toast.LENGTH_LONG).show();
        } 
        
        if (notifyMsg != null) {
            String title = (notifyTitle != null) ? notifyTitle : "AZenith";
            boolean chrono = "true".equals(useChrono);
            long timeout = (timeoutStr != null) ? Long.parseLong(timeoutStr) : 0;
            showNotification(title, notifyMsg, chrono, timeout);
        }
    
        finish();
    }


    private void showNotification(String title, String message, boolean chrono, long timeoutMs) {
        NotificationManager manager = (NotificationManager) getSystemService(NOTIFICATION_SERVICE);
        
        boolean isProfile = title.toLowerCase().contains("profile") || title.toLowerCase().contains("mode") || title.toLowerCase().contains("initializing...");
        String currentChannel = isProfile ? CH_PROFILE : CH_SYSTEM;

        // Buat Channel
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                currentChannel, 
                isProfile ? "AZenith Profiles" : "AZenith System", 
                NotificationManager.IMPORTANCE_LOW // LOW agar tidak menimpa notif lain secara agresif
            );
            manager.createNotificationChannel(channel);
        }

        int notificationId = isProfile ? PROFILE_ID : title.hashCode();

        Notification.Builder builder;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            builder = new Notification.Builder(this, currentChannel);
        } else {
            builder = new Notification.Builder(this);
        }

        builder.setSmallIcon(getApplicationInfo().icon)
               .setContentTitle(title)
               .setContentText(message)
               .setUsesChronometer(chrono)
               .setAutoCancel(!isProfile);

        // TRUE PERSISTENT: Agar benar-benar tidak bisa dihapus
        if (isProfile) {
            builder.setOngoing(true); 
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                builder.setPriority(Notification.PRIORITY_MAX);
            }
        }

        if (timeoutMs > 0 && Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            builder.setTimeoutAfter(timeoutMs);
        }

        Notification notification = (Build.VERSION.SDK_INT >= Build.VERSION_CODES.JELLY_BEAN) 
                                    ? builder.build() : builder.getNotification();

        // Tambahan Flag Paksa agar tidak bisa di-clear manual
        if (isProfile) {
            notification.flags |= Notification.FLAG_ONGOING_EVENT;
            notification.flags |= Notification.FLAG_NO_CLEAR;
        }

        manager.notify(notificationId, notification);

        if (timeoutMs > 0 && Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            new Handler().postDelayed(() -> manager.cancel(notificationId), timeoutMs);
        }
    }
}
