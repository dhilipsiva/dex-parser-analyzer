package fixtures.positive;

import android.webkit.WebSettings;

/** M7_WEBVIEW_FILE_ACCESS fires: JavaScript enabled and file access allowed. */
public class WebViewFileAccess {
    public static void configure(WebSettings settings) {
        settings.setJavaScriptEnabled(true);
        settings.setAllowFileAccess(true);
    }
}
