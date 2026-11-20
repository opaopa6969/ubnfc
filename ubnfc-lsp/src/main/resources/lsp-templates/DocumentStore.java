package __PACKAGE__;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;

/**
 * 開いている文書の本文と、その本文に対する解析結果。
 * full sync（LSP {@code TextDocumentSyncKind.Full}）なので版ごとに丸ごと差し替える。
 *
 * <p>解析は 1 つの本文につき 1 回だけ行い、definition / hover / semantic tokens などの要求は
 * 同じ {@link Analysis} を読む（増分解析は無い＝D-060、が同じ本文を 2 度解析することもしない）。
 */
public final class DocumentStore {
    record Snapshot(Analysis analysis, Integer version, long revision) {
        String diagnosticId() { return Long.toString(revision); }
    }

    private final Map<String, Snapshot> documents = new ConcurrentHashMap<>();
    private final AtomicLong revisions = new AtomicLong();

    public void put(String uri, Analysis analysis) { put(uri, analysis, null); }

    Snapshot put(String uri, Analysis analysis, Integer version) {
        var snapshot = new Snapshot(analysis, version, revisions.incrementAndGet());
        documents.put(uri, snapshot);
        return snapshot;
    }

    Snapshot snapshot(String uri) { return documents.get(uri); }
    public void remove(String uri) { documents.remove(uri); }

    public Analysis get(String uri) {
        var document = snapshot(uri);
        return document == null ? null : document.analysis();
    }
    public String text(String uri) {
        Analysis analysis = get(uri);
        return analysis == null ? null : analysis.text();
    }
    public java.util.Set<String> uris() { return documents.keySet(); }
}
