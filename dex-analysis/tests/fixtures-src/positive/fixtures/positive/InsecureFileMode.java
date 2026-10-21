package fixtures.positive;

import android.content.Context;

/** M9_INSECURE_FILE_MODE fires: a world-readable file. */
public class InsecureFileMode {
    public static void write(Context context) throws Exception {
        context.openFileOutput("state", Context.MODE_WORLD_READABLE).close();
    }
}
