package fixtures.negative;

import android.util.Log;

/** SNK_LOG_ERROR must not fire: a constant message. */
public class LogErrorLeak {
    public static void log() {
        Log.e("fixture", "failed");
    }
}
