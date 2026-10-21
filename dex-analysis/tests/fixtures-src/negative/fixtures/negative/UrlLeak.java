package fixtures.negative;

import java.net.URL;

/** SNK_URL_CONSTRUCTOR must not fire: a constant URL. */
public class UrlLeak {
    public static URL fixed() throws Exception {
        return new URL("https://example.com/");
    }
}
