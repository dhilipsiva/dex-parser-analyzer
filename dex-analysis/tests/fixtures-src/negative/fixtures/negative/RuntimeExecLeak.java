package fixtures.negative;

/** SNK_RUNTIME_EXEC must not fire: a constant command. */
public class RuntimeExecLeak {
    public static Process run() throws Exception {
        return Runtime.getRuntime().exec("ls");
    }
}
