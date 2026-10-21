package fixtures.negative;

import android.webkit.WebSettings;

/** M7_WEBVIEW_FILE_ACCESS must not fire: JavaScript is explicitly disabled. */
public class WebViewFileAccess {
    public static void configure(WebSettings settings) {
        settings.setJavaScriptEnabled(false);
        settings.setAllowFileAccess(true);
    }
}
