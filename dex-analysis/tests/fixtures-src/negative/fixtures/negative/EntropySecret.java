package fixtures.negative;

/**
 * M1_HARDCODED_SECRET must not fire: text with high character entropy that is not a token (an
 * identifier with digits, a resource-style name, a constant name, a transformation, SQL, a URL and
 * a file path).
 */
public class EntropySecret {
    public static String[] values() {
        return new String[] {
            "TaskStackBuilderApi16Impl",
            "Base_V7_Theme_AppCompat_Dialog",
            "ACTION_MEDIA_BUTTON_V2_COMPAT",
            "DES/ECB/PKCS5Padding",
            "SELECT id, name FROM users WHERE id = 42",
            "https://api.example.com/v1/items?id=42",
            "res/layout/activity_main2.xml",
        };
    }
}
