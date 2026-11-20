package org.unlaxer.ubnfc.examples.p4.generated.internal;

import java.util.List;

/**
 * 認識が完了した規則の出来事。capture は認識順のまま保持し、field 列へ並べ替えない。
 * 最終的に選択された出来事からだけ typed AST を構築するので、捨てた枝の変換失敗や
 * node span は公開されない。memo はこの不変な出来事をそのまま再生できる。
 */
public class AstEvent {
    private final String id, fallback;
    private final int start, end;
    final Session.Fields fields;
    final List<Session.Occurrence> captures;
    static final Object UNBUILT = new Object();
    Object built = UNBUILT;
    private List<AstEvent> self;
    private Value value;
    private List<Value> selfValues;

    public AstEvent(String id, int start, int end, Session.Fields fields,
                    List<Session.Occurrence> captures, String fallback) {
        this.id = id; this.start = start; this.end = end;
        this.fields = fields; this.captures = captures; this.fallback = fallback;
    }
    public static AstEvent leaf(String id, int start, int end) {
        return new AstEvent(id, start, end, Session.Fields.NONE, List.of(), null);
    }
    public String id() { return id; }
    public int start() { return start; }
    public int end() { return end; }
    public String fallback() { return fallback; }
    public List<AstEvent> self() { List<AstEvent> result = self; if (result == null) self = result = List.of(this); return result; }
    public Value value() { Value result = value; if (result == null) value = result = new Value(start, end, self()); return result; }
    public List<Value> values() { List<Value> result = selfValues; if (result == null) selfValues = result = List.of(value()); return result; }
    /** 要素は透明な helper の意味値を enclosing capture の範囲と分けて保持する。 */
    public record Value(int start, int end, List<AstEvent> nodes, String text, List<Value> elements) {
        public Value(int start, int end, List<AstEvent> nodes, String text) { this(start, end, nodes, text, null); }
        public Value(int start, int end, List<AstEvent> nodes) { this(start, end, nodes, null); }
    }
}
