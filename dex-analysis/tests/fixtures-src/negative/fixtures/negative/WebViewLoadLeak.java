package fixtures.negative;

import android.webkit.WebView;

/** SNK_WEBVIEW_LOAD_URL must not fire: a constant URL. */
public class WebViewLoadLeak {
    public static void load(WebView view) {
        view.loadUrl("https://example.com/");
    }
}
