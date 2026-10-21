package fixtures.positive;

import javax.crypto.Cipher;

/** M10_CIPHER fires at High: DES in ECB mode. */
public class WeakCipher {
    public static Cipher create() throws Exception {
        return Cipher.getInstance("DES/ECB/PKCS5Padding");
    }
}
