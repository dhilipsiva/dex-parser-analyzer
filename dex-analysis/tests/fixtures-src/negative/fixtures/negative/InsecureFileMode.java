package fixtures.negative;

import android.content.Context;

/** M9_INSECURE_FILE_MODE must not fire: a private file. */
public class InsecureFileMode {
    public static void write(Context context) throws Exception {
        context.openFileOutput("state", Context.MODE_PRIVATE).close();
    }
}
