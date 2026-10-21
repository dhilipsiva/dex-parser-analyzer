package fixtures.negative;

/** M1_RUNTIME_EXEC must not fire: Runtime is used, but nothing is executed. */
public class RuntimeExec {
    public static int cores() {
        return Runtime.getRuntime().availableProcessors();
    }
}
