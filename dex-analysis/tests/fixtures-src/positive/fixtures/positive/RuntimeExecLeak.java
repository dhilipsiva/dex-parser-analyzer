package fixtures.positive;

import android.content.Intent;

/** SNK_RUNTIME_EXEC fires: an Intent extra executed as a command. */
public class RuntimeExecLeak {
    public static Process leak(Intent intent) throws Exception {
        return Runtime.getRuntime().exec(intent.getStringExtra("command"));
    }
}
