package fixtures.negative;

import android.database.sqlite.SQLiteDatabase;

/** SNK_SQLITE_EXEC must not fire: constant SQL. */
public class SqliteExecLeak {
    public static void migrate(SQLiteDatabase db) {
        db.execSQL("CREATE TABLE notes (id INTEGER)");
    }
}
