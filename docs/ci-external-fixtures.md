# CI の外部 fixture と pin

ubnfc のテストの一部は、別 repository にある文法・corpus・spec-corpus を fixture として読む。

| repository | 読む主なもの | 変数（未設定時の既定） |
|---|---|---|
| [unlaxer-parser](https://github.com/opaopa6969/unlaxer-parser) | `unlaxer-dsl/grammar/ubnf.ubnf` ほか golden 対象の `.ubnf`、`rust/unlaxer-ubnf/tests/fixtures/`、spec-corpus、corpus | `UNLAXER_PARSER_DIR` / `UNLAXER_PARSER_ROOT`（`../unlaxer-parser`） |
| [tinyexpression](https://github.com/opaopa6969/tinyexpression) | `tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf` ほか | `TINYEXPRESSION_DIR`（`../tinyexpression`） |

## CI での取得（issue #105）

CI（`.github/workflows/ci.yml` の native job）は、[`scripts/external-pins.env`](../scripts/external-pins.env)
に書いた **commit** だけを取得して読む。runner マシン上の開発者の作業ツリーは読まない。

1. `scripts/fetch-external-fixtures.sh "$RUNNER_TEMP/external"` が、各 repository を
   `git fetch --depth 1 origin <commit>` で取得し、detached HEAD で展開する。
2. 同じスクリプトが `UNLAXER_PARSER_DIR` / `UNLAXER_PARSER_ROOT` / `TINYEXPRESSION_DIR` を
   `$GITHUB_ENV` へ書き、後続の step（`cargo test --workspace`）に渡す。
3. 変数を読まずに sibling の相対パスで探すテストのために、`_work/ubnfc/` の sibling にも
   同じ checkout への symlink を張る。

このため CI の結果は ubnfc 側の commit（と pin ファイル）だけで決まる。golden
（`ir/fixtures/*.ir.json`）が記録する `sources[].sha256` は、pin した commit の文法と一致している。

## 手元で CI と同じ fixture を使う

```sh
eval "$(scripts/fetch-external-fixtures.sh "$(mktemp -d)" | sed 's/^/export /')"
cargo test --workspace --no-fail-fast
```

変数を設定しなければ、これまでどおり sibling（`../unlaxer-parser` / `../tinyexpression`）を読む。

## pin を更新する手順

pin の更新は**1 PR で 1 回**、追従に必要な ubnfc 側の変更（golden の再生成・テストの期待値・
front の構文対応）と同じ PR に入れる。pin だけを先に上げると CI が落ちる。

1. 上げ先の commit を決める。原則は各 repository の `master` の merge commit。
   ```sh
   git ls-remote https://github.com/opaopa6969/unlaxer-parser.git refs/heads/master
   git ls-remote https://github.com/opaopa6969/tinyexpression.git refs/heads/master
   ```
2. `scripts/external-pins.env` の `*_COMMIT` を **40 桁の完全な SHA** に書き換え、コメント
   （何の merge か・日付）も直す。短縮形・branch 名・tag 名はスクリプトが拒否する。
3. 新しい pin で取得し、全テストを回す。
   ```sh
   dir=$(mktemp -d)
   eval "$(scripts/fetch-external-fixtures.sh "$dir" | sed 's/^/export /')"
   cargo test --workspace --no-fail-fast
   ```
4. 落ちたものを直す。よくあるのは次のとおり。
   - **golden の sha256 / 内容の差**（`pipeline_golden.rs`）: 文法が変わっただけで ubnfc の挙動が
     正しいなら golden を再生成する。
     `ir` は `--grammar` に渡した相対パスを `sources[].path` に書くので、`pipeline_golden.rs` の
     `run` と同じく fixture repository の root へ移って相対パスで渡す（例は `ubnf.ubnf`）。
     ```sh
     cargo build -p ubnfc-front --bin ubnfc
     ubnfc=$PWD/target/debug/ubnfc first=$PWD/scanners/first-chars.json
     (cd "$UNLAXER_PARSER_DIR" && "$ubnfc" ir --grammar unlaxer-dsl/grammar/ubnf.ubnf \
         --extern-first-chars "$first") > ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json
     ```
   - **ubnfc front が受理しない新構文**: front 側の対応が先。その PR が merge されるまで pin は上げない。
   - **件数の固定値**（規則数・corpus 件数など）: 変化の理由を確かめてから期待値を直す。
5. PR 本文に「旧 pin → 新 pin」と、各 repository の間の差分（`git log --oneline 旧..新 -- <読むパス>`）の
   要点、直したテストを書く。

## 外部 pin から切り離した IR（`ir/frozen/`）

backend の回帰テスト・commit 済みの生成物（`examples/p4-rust` など）・ベンチマークは、外部文法の
変化とは無関係に同じ文法を使う。そのため `ir/fixtures/` の golden ではなく `ir/frozen/` の固定 IR を
読む（[`ir/frozen/README.md`](../ir/frozen/README.md)）。pin を上げて `ir/fixtures/` の golden が
変わっても、これらは動かない。2026-11-01 の更新（issue #8）で、tinyexpression P4 の旧版（`f86ce8a5`）を
ここへ移した。

## 注意

- `scripts/external-pins.env` は `KEY=値` だけの単純な形式で、スクリプトは `source` せずに 1 行ずつ
  検査して読む（空白・引用符・`$` を含む行は拒否）。
- 取得は 2 repository 合わせて約 90 MB（depth 1）。`$RUNNER_TEMP` は job ごとに runner が片付ける。
- 外部 repository の `.ubnf` はコメントを変えただけでも sha256 が変わり、golden が落ちる。
