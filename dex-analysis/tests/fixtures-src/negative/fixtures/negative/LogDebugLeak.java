package fixtures.negative;

import android.util.Log;

/** SNK_LOG_DEBUG must not fire: a constant message. */
public class LogDebugLeak {
    public static void log() {
        Log.d("fixture", "started");
    }
}
