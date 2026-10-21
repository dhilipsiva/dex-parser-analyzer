package fixtures.positive;

import android.content.Intent;
import android.util.Log;

/** SNK_LOG_DEBUG fires: an Intent extra written to Log.d. */
public class LogDebugLeak {
    public static void leak(Intent intent) {
        Log.d("fixture", intent.getStringExtra("extra"));
    }
}
