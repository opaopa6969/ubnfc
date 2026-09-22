package target.internal;

/** count を増やした後に呼ぶ。終端検査・子失敗による停止は呼出し側で先に行う。 */
public final class RepeatProgress {
    private RepeatProgress() {}
    public static boolean more(int count, int startConsumed, int endConsumed, int max) {
        return startConsumed != endConsumed && count < max;
    }
    public static boolean accepts(int count, int min, int max) {
        return min <= count && count <= max;
    }
}
