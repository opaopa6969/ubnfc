package generated.internal;

import java.util.List;

/** A/B と同値検証専用の参照経路。認識中に field 列を materialize する旧方式。 */
public final class Recipe extends AstEvent {
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static final List<Value>[] EMPTY = new List[0];
    private final List<Value>[] values;
    public Recipe(String id, int start, int end, List<Value>[] values, String fallback) {
        super(id, start, end, Session.Fields.NONE, List.of(), fallback);
        this.values = values;
    }
    /** 旧実装と同じ共有空配列。参照経路だけに余計な確保を加えない。 */
    public static Recipe leaf(String id, int start, int end) { return new Recipe(id, start, end, EMPTY, null); }
    public List<Value> fieldValues(int index) { return values[index]; }
}
