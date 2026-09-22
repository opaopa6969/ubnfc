package org.ubnfc.p4;
import static org.junit.jupiter.api.Assertions.*;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.ubnfc.p4.generated.api.Json;

/** 実行できた旧生成物の snapshot。引用符・UTF-16 span の差を隠さない。 */
class LegacySnapshotTest {
    @Test void basicAndMultilineMatchLegacyExactly() throws Exception {
        for (String name : new String[]{"valid-basic", "valid-multiline"})
            assertEquals(Files.readString(Path.of("src/test/resources/legacy/legacy-" + name + ".json")),
                Files.readString(Path.of("src/test/resources/golden/" + name + ".tiny.json")));
    }
    @Test void unicodeDifferenceRemainsVisible() throws Exception {
        var oldTree = (Map<?, ?>) Json.read(Files.readString(Path.of("src/test/resources/legacy/legacy-valid-unicode.json")));
        var newTree = (Map<?, ?>) Json.read(Files.readString(Path.of("src/test/resources/golden/valid-unicode.tiny.json")));
        assertNotEquals(oldTree.get("span"), newTree.get("span"));
        assertNotEquals(oldTree, newTree);
    }
}
