package fixtures.negative;

/** M1_PROCESS_BUILDER must not fire: the class is only named, never constructed. */
public class ProcessBuilderExec {
    public static String name() {
        return ProcessBuilder.class.getName();
    }
}
