package fixtures.negative;

import javax.crypto.Cipher;

/** M10_CIPHER must not fire: AES/GCM, next to a string that merely contains 'des'. */
public class WeakCipher {
    public static Cipher create() throws Exception {
        System.out.println("description");
        return Cipher.getInstance("AES/GCM/NoPadding");
    }
}
