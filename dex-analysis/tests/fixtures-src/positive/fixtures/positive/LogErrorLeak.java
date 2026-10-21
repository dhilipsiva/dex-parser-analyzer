package fixtures.positive;

import android.content.Intent;
import android.util.Log;

/** SNK_LOG_ERROR fires: an Intent extra written to Log.e. */
public class LogErrorLeak {
    public static void leak(Intent intent) {
        Log.e("fixture", intent.getStringExtra("extra"));
    }
}
