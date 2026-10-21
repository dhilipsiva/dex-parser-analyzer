package fixtures.positive;

/** M1_PROCESS_BUILDER fires: a ProcessBuilder is constructed. */
public class ProcessBuilderExec {
    public static Process run() throws Exception {
        return new ProcessBuilder("ls").start();
    }
}
