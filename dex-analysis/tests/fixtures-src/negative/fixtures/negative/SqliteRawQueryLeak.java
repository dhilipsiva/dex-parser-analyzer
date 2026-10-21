package fixtures.negative;

import android.database.sqlite.SQLiteDatabase;

/** SNK_SQLITE_RAW_QUERY must not fire: a constant query. */
public class SqliteRawQueryLeak {
    public static void count(SQLiteDatabase db) {
        db.rawQuery("SELECT COUNT(*) FROM notes", null);
    }
}
