package fixtures.positive;

import android.content.Intent;
import android.database.sqlite.SQLiteDatabase;

/** SNK_SQLITE_RAW_QUERY fires: an Intent extra used as a raw query. */
public class SqliteRawQueryLeak {
    public static void leak(Intent intent, SQLiteDatabase db) {
        db.rawQuery(intent.getStringExtra("query"), null);
    }
}
