package fixtures.positive;

import android.webkit.WebSettings;
import android.webkit.WebView;

/** M7_WEBVIEW_JS_INTERFACE fires: JavaScript enabled and an interface attached. */
public class WebViewJsInterface {
    public static void configure(WebView view, WebSettings settings, Object bridge) {
        settings.setJavaScriptEnabled(true);
        view.addJavascriptInterface(bridge, "bridge");
    }
}
