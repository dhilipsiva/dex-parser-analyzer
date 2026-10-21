package fixtures.positive;

import android.content.Intent;
import java.net.URL;

/** SNK_URL_CONSTRUCTOR fires: an Intent extra becomes a URL. */
public class UrlLeak {
    public static URL leak(Intent intent) throws Exception {
        return new URL(intent.getStringExtra("target"));
    }
}
