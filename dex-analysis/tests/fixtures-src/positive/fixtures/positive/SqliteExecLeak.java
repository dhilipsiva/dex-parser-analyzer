package fixtures.positive;

import android.content.Intent;
import android.database.sqlite.SQLiteDatabase;

/** SNK_SQLITE_EXEC fires: an Intent extra executed as SQL. */
public class SqliteExecLeak {
    public static void leak(Intent intent, SQLiteDatabase db) {
        db.execSQL(intent.getStringExtra("query"));
    }
}
