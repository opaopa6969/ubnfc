# M3 oracle 文法の正規化 IR

`corpus/oracle/{enum*,error,eval*,recovery*,typeof}.json` の `grammar` を
`ubnfc` front（基点 `6829efe`）で出力した16 IR。oracle 本体は変更していない。
sourceId は入力ファイルの basename なので、一時ディレクトリに依存しない。
`eval-sum` は仕様どおり front が E-LOWER-SUM-EVAL で拒否するため、正常 IR は置かない。
その拒否は M3FeatureTest で和型への自動 eval を設定した IR によっても固定する。

再採取の手順（repo root、各 NAME は保存ファイルの basename）:

```sh
m3_scratch=$(mktemp -d)
python3 - "$m3_scratch" <<'PY'
import json, pathlib, sys
scratch = pathlib.Path(sys.argv[1])
for path in pathlib.Path('ubnfc-java/src/test/resources/ir/m3').glob('*.ir.json'):
    name = path.name.removesuffix('.ir.json')
    oracle = json.loads(pathlib.Path(f'corpus/oracle/{name}.json').read_text())
    (scratch / f'{name}.ubnf').write_text(oracle['grammar'])
PY
# NAME ごとに実行し、既存 fixture との差分を確認する。
cargo run -q --bin ubnfc -- ir --grammar "$m3_scratch/NAME.ubnf" > "$m3_scratch/NAME.ir.json"
```

AUTO/SKIP の fixture は front の旧 SYNC lowering もそのまま保存する。
backend は legacyMarker と注釈の組合せに限定して mode と literal FOLLOW を補正する。
`M3RecoveryTest` は明示 AUTO/SKIP の IR も別に作り、補正に依存しない実行経路を検証する。
