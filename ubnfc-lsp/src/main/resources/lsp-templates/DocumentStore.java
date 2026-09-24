package __PACKAGE__;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * 開いている文書の本文と、その本文に対する解析結果。
 * full sync（LSP {@code TextDocumentSyncKind.Full}）なので版ごとに丸ごと差し替える。
 *
 * <p>解析は 1 つの本文につき 1 回だけ行い、definition / hover / semantic tokens などの要求は
 * 同じ {@link Analysis} を読む（増分解析は無い＝D-060、が同じ本文を 2 度解析することもしない）。
 */
public final class DocumentStore {
    private final Map<String, Analysis> documents = new ConcurrentHashMap<>();

    public void put(String uri, Analysis analysis) { documents.put(uri, analysis); }
    public void remove(String uri) { documents.remove(uri); }
    public Analysis get(String uri) { return documents.get(uri); }
    public String text(String uri) {
        Analysis analysis = documents.get(uri);
        return analysis == null ? null : analysis.text();
    }
    public java.util.Set<String> uris() { return documents.keySet(); }
}
