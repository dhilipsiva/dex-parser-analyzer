package fixtures.positive;

/** M5_INSECURE_HTTP fires: a cleartext http:// endpoint. */
public class InsecureHttp {
    public static String endpoint() {
        return "http://api.example.com/login";
    }
}
