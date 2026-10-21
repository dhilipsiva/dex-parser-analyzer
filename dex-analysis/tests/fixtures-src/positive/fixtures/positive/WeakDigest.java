package fixtures.positive;

import java.security.MessageDigest;

/** M10_DIGEST fires at High: MD5. */
public class WeakDigest {
    public static MessageDigest create() throws Exception {
        return MessageDigest.getInstance("MD5");
    }
}
