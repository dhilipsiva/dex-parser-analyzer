package fixtures.negative;

/** M5_INSECURE_HTTP must not fire: the endpoint uses https. */
public class InsecureHttp {
    public static String endpoint() {
        return "https://api.example.com/login";
    }
}
