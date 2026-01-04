package zx.azenith;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.os.Handler;
import android.os.Looper;

public class MyReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent intent) {
        if ("RE-SHOW_NOTIF".equals(intent.getAction())) {
            boolean isProfile = intent.getBooleanExtra("isProfile", false);
            if (isProfile) {
                String title = intent.getStringExtra("title");
                String msg = intent.getStringExtra("message");
                // Ambil status chrono asli dari intent yang baru saja di-swipe
                boolean chronoStatus = intent.getBooleanExtra("chrono_bool", false);
    
                new Handler(Looper.getMainLooper()).postDelayed(() -> {
                    Intent reshow = new Intent(context, MainActivity.class);
                    // Flag agar Activity segar kembali
                    reshow.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
                    
                    reshow.putExtra("notifytitle", title);
                    reshow.putExtra("notifytext", msg);
                    // Kembalikan ke format String agar dibaca "true".equals(useChrono)
                    reshow.putExtra("chrono", String.valueOf(chronoStatus)); 
                    
                    context.startActivity(reshow);
                }, 3000);
            }
        }
    }
}
