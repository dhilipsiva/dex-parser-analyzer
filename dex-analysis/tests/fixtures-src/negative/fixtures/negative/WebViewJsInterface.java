package fixtures.negative;

import android.webkit.WebSettings;
import android.webkit.WebView;

/** M7_WEBVIEW_JS_INTERFACE must not fire: JavaScript is explicitly disabled. */
public class WebViewJsInterface {
    public static void configure(WebView view, WebSettings settings, Object bridge) {
        settings.setJavaScriptEnabled(false);
        view.addJavascriptInterface(bridge, "bridge");
    }
}
