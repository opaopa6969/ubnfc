package target.api;

/** 採用枝のみ。リスト順が capture の完了順。 */
public record Capture(int siteId, String name, Span span) {}
