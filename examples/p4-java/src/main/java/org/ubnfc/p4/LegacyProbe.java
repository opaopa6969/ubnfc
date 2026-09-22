package org.ubnfc.p4;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Optional;
import org.ubnfc.p4.generated.api.Json;

/** 旧生成物を反射で調査する任意ツール。通常のビルドに unlaxer 依存を追加しない。 */
public final class LegacyProbe {
    private static Method span;
    public static void main(String[] args) throws Exception {
        for (String name : List.of("org.unlaxer.parser.AbstractParser", "org.unlaxer.context.ParseContext",
                "org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper")) {
            Class<?> type = Class.forName(name);
            System.out.println("読み込み元 " + name + " " + type.getProtectionDomain().getCodeSource().getLocation());
        }
        Class<?> mapper = Class.forName("org.unlaxer.tinyexpression.generated.p4.TinyExpressionP4Mapper");
        Class<?> facade = Class.forName("org.unlaxer.tinyexpression.p4.P4PreferredAstMapper");
        span = mapper.getMethod("sourceSpanOf", Object.class);
        String text = Files.readString(Path.of(args[1]));
        long start = System.nanoTime();
        try {
            Object ast;
            if (args[0].equals("mapper")) ast = mapper.getMethod("parse", String.class).invoke(null, text);
            else {
                Object result = facade.getMethod("parseDetailed", String.class).invoke(null, text);
                ast = result.getClass().getMethod("ast").invoke(result);
            }
            System.out.printf("成功 %s %.3f ms%n", args[1], (System.nanoTime() - start) / 1e6);
            String json = Json.write(canonical(ast));
            if (args.length > 2) Files.writeString(Path.of(args[2]), json + "\n");
            else System.out.println(json);
        } catch (InvocationTargetException failure) {
            System.out.printf("失敗 %s %.3f ms%n", args[1], (System.nanoTime() - start) / 1e6);
            failure.getCause().printStackTrace(System.out);
            System.exit(1);
        }
    }
    private static Object canonical(Object value) throws Exception {
        if (value instanceof Optional<?> opt) return canonical(opt.orElse(null));
        if (value instanceof List<?> list) {
            var result = new java.util.ArrayList<Object>();
            for (Object item : list) result.add(canonical(item));
            return result;
        }
        if (value == null || !value.getClass().isRecord()) return value;
        var fields = new LinkedHashMap<String, Object>();
        for (var component : value.getClass().getRecordComponents())
            fields.put(component.getName(), canonical(component.getAccessor().invoke(value)));
        var result = new LinkedHashMap<String, Object>();
        result.put("type", value.getClass().getSimpleName());
        var range = (Optional<?>) span.invoke(null, value);
        if (range.isPresent()) {
            int[] offsets = (int[]) range.get();
            result.put("span", List.of(offsets[0], offsets[1]));
        } else result.put("span", null);
        result.put("fields", fields);
        return result;
    }
}
