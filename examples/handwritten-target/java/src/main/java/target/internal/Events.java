package target.internal;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import target.api.Capture;
import target.api.Span;
import target.rt.Input;

/** 永続 concat 列。失敗枝は参照を捨てるだけ、memo hit は同じ列を再採用できる。 */
public final class Events {
    private final Events left, right;
    private final int site, start, end;
    private final String name;

    private Events(Events left, Events right, int site, String name, int start, int end) {
        this.left = left; this.right = right;
        this.site = site; this.name = name; this.start = start; this.end = end;
    }

    public static Events capture(int site, String name, int start, int end) {
        return new Events(null, null, site, name, start, end);
    }

    public static Events concat(Events left, Events right) {
        if (left == null) return right;
        if (right == null) return left;
        return new Events(left, right, -1, null, 0, 0);
    }

    public static List<Capture> materialize(Events events, Input input) {
        var result = new ArrayList<Capture>();
        var stack = new ArrayDeque<Events>();
        if (events != null) stack.push(events);
        while (!stack.isEmpty()) {
            var next = stack.pop();
            if (next.site >= 0) {
                result.add(new Capture(next.site, next.name,
                    new Span(input.cpIndex(next.start), input.cpIndex(next.end))));
            } else {
                stack.push(next.right);
                stack.push(next.left);
            }
        }
        return List.copyOf(result);
    }
}
