package fixtures.negative;

import java.security.MessageDigest;

/** M10_DIGEST must not fire: SHA-256. */
public class WeakDigest {
    public static MessageDigest create() throws Exception {
        return MessageDigest.getInstance("SHA-256");
    }
}
