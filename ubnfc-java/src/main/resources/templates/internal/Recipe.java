package generated.internal;

import java.util.List;

/** A/B と同値検証専用の参照経路。認識中に field 列を materialize する旧方式。 */
public final class Recipe extends AstEvent {
    private final List<Value>[] values;
    public Recipe(String id, int start, int end, List<Value>[] values, String fallback) {
        super(id, start, end, Session.Fields.NONE, List.of(), fallback);
        this.values = values;
    }
    public List<Value> fieldValues(int index) { return values[index]; }
}
