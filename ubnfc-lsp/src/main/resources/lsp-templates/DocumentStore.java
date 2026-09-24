package __PACKAGE__;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/** 開いている文書の本文。full sync（LSP TextDocumentSyncKind.Full）なので版ごとに丸ごと差し替える。 */
public final class DocumentStore {
    private final Map<String, String> documents = new ConcurrentHashMap<>();

    public void put(String uri, String text) { documents.put(uri, text); }
    public void remove(String uri) { documents.remove(uri); }
    public String get(String uri) { return documents.get(uri); }
    public java.util.Set<String> uris() { return documents.keySet(); }
}
