package fixtures.positive;

/** M1_RUNTIME_EXEC fires: Runtime.exec. */
public class RuntimeExec {
    public static Process run() throws Exception {
        return Runtime.getRuntime().exec("ls");
    }
}
