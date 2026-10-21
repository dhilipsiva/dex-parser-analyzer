package fixtures.positive;

import android.content.Intent;
import android.webkit.WebView;

/** SNK_WEBVIEW_LOAD_URL fires: an Intent extra loaded into a WebView. */
public class WebViewLoadLeak {
    public static void leak(Intent intent, WebView view) {
        view.loadUrl(intent.getStringExtra("target"));
    }
}
