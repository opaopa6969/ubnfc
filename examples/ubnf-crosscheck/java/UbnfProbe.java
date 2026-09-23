// UBNF 4 実装比較のための Java bootstrap プローブ。
// unlaxer-dsl の UBNFMapper（手書き combinator の UBNFParsers を駆動する）を
// 1 ファイルずつ呼び、rust/unlaxer-ubnf の tests/support/canonical.rs と
// 同じ span 抜きの正準 JSON を出す。差分を消す正規化は RuleBody の
// SequenceBody/ChoiceBody 吸収だけで、これは RustUbnfFrontendConformanceTest と同一。
import java.lang.reflect.RecordComponent;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.unlaxer.dsl.bootstrap.UBNFAST;
import org.unlaxer.dsl.bootstrap.UBNFMapper;

public final class UbnfProbe {
    public static void main(String[] arguments) throws Exception {
        for (String path : arguments) {
            String source = Files.readString(Path.of(path));
            StringBuilder out = new StringBuilder();
            out.append("{\"input\":").append(quote(path));
            try {
                UBNFAST.UBNFFile file = UBNFMapper.parse(source);
                out.append(",\"ok\":true,\"ast\":").append(canonical(file));
            } catch (Throwable error) {
                Throwable cause = error;
                out.append(",\"ok\":false,\"error\":")
                   .append(quote(cause.getClass().getName() + ": " + String.valueOf(cause.getMessage())));
            }
            out.append("}");
            System.out.println(out);
        }
    }

    static String canonical(Object value) throws Exception {
        if (value == null) return "null";
        if (value instanceof Optional<?> optional) return canonical(optional.orElse(null));
        if (value instanceof String text) return quote(text);
        if (value instanceof Character character) return quote(String.valueOf(character));
        if (value instanceof Number number) return number.toString();
        if (value instanceof Boolean flag) return flag.toString();
        if (value instanceof Enum<?> constant) return quote(constant.name());
        if (value instanceof List<?> list) {
            StringBuilder out = new StringBuilder("[");
            for (int i = 0; i < list.size(); i++) {
                if (i > 0) out.append(',');
                out.append(canonical(list.get(i)));
            }
            return out.append(']').toString();
        }
        if (value instanceof Map<?, ?> map) {
            StringBuilder out = new StringBuilder("{");
            boolean first = true;
            for (Map.Entry<?, ?> entry : map.entrySet()) {
                if (!first) out.append(',');
                first = false;
                out.append(quote(entry.getKey().toString())).append(':').append(canonical(entry.getValue()));
            }
            return out.append('}').toString();
        }
        // 後置 ?/* は Java では合成 SequenceBody、明示括弧は ChoiceBody。両者を吸収する。
        if (value instanceof UBNFAST.RuleBody body) {
            StringBuilder out = new StringBuilder("[\"body\",[");
            if (body instanceof UBNFAST.ChoiceBody choice) {
                List<UBNFAST.SequenceBody> alternatives = choice.alternatives();
                for (int i = 0; i < alternatives.size(); i++) {
                    if (i > 0) out.append(',');
                    out.append(canonical(alternatives.get(i).elements()));
                }
            } else {
                out.append(canonical(((UBNFAST.SequenceBody) body).elements()));
            }
            return out.append("]]").toString();
        }
        if (!value.getClass().isRecord()) {
            throw new IllegalStateException("record ではありません: " + value.getClass());
        }
        StringBuilder out = new StringBuilder("[").append(quote(value.getClass().getSimpleName()));
        for (RecordComponent component : value.getClass().getRecordComponents()) {
            out.append(',').append(canonical(component.getAccessor().invoke(value)));
        }
        return out.append(']').toString();
    }

    static String quote(String text) {
        StringBuilder out = new StringBuilder("\"");
        text.codePoints().forEach(c -> {
            switch (c) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                default -> {
                    if (c < 0x20) out.append(String.format("\\u%04x", c));
                    else out.appendCodePoint(c);
                }
            }
        });
        return out.append('"').toString();
    }
}
