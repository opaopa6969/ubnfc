use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn cli(file: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stax"))
        .args(args)
        .arg(file)
        .output()
        .unwrap()
}
fn success(output: Output, want: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), want);
    assert!(output.stderr.is_empty());
}
fn failure(output: Output, reason: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "失敗時は途中の stdout を公開しない"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(reason),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
struct Files(PathBuf);
impl Files {
    fn new(files: &[(&str, &str)]) -> Self {
        let out = Command::new("mktemp").arg("-d").output().unwrap();
        assert!(out.status.success());
        let dir = PathBuf::from(String::from_utf8(out.stdout).unwrap().trim());
        for (name, text) in files {
            fs::write(dir.join(name), text).unwrap();
        }
        Self(dir)
    }
    fn run(&self, args: &[&str]) -> Output {
        cli(&self.0.join("main.stax"), args)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(source: &str) -> Output {
    Files::new(&[("main.stax", source)]).run(&[])
}

#[test]
fn twenty_samples_have_fixed_outputs_or_explained_type_errors() {
    let dir = root().join("examples/stax/samples");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).unwrap()).unwrap();
    let cases = manifest["samples"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for case in cases {
        let path = dir.join(case["file"].as_str().unwrap());
        if let Some(reason) = case["errorContains"].as_str() {
            failure(cli(&path, &[]), reason);
        } else {
            success(cli(&path, &["--check"]), "");
            let via = case["runVia"].as_str();
            if via.is_some() {
                failure(cli(&path, &[]), "main が必要");
            }
            success(
                cli(&via.map(|p| dir.join(p)).unwrap_or(path), &[]),
                case["stdout"].as_str().unwrap(),
            );
        }
    }
}
#[test]
fn nominal_structs_aliases_and_persistent_updates() {
    success(run("type M = Int ; struct A { x: M } : main ( -- ) 1 A dup 2 !x .x int_to_str print \" \" print .x int_to_str print ;"),"2 1");
    success(run("struct A { x: Int } struct B { x: Str } : main ( -- ) 3 A .x int_to_str print \"b\" B .x print ;"),"3b");
    failure(
        run("struct A { x: Int } struct B { x: Int } : wrong ( A -- B ) ; : main ( -- ) ;"),
        "型不一致",
    );
    failure(
        run("struct A { x: Int } : main ( -- ) 1 A \"bad\" !x drop ;"),
        "型不一致",
    );
    failure(
        run("struct A { x: Int } : main ( -- ) 1 A .y drop ;"),
        "未知のフィールド",
    );
}
#[test]
fn stack_depth_types_effects_and_enum_comparisons() {
    for (source, reason) in [
        (": main ( -- ) drop ;", "スタック深さ"),
        (": main ( -- ) 1 true + drop ;", "型不一致"),
        (": broken ( -- Int ) ; : main ( -- ) ;", "宣言効果"),
        (
            "type C = | Red ; type D = | Cold ; : main ( -- ) Red Cold = drop ;",
            "型不一致",
        ),
        (
            "struct A { x: Int } : main ( -- ) 1 A dup = drop ;",
            "等価比較",
        ),
        (": first ( -- ) later ; : later ( -- ) ;", "未定義の語"),
        (": self ( -- ) self ;", "未定義の語"),
    ] {
        failure(run(source), reason);
    }
    success(
        run("type C = | Red | Blue ; : main ( -- ) Red Red = [ \"yes\" print ] [ ] choose call ;"),
        "yes",
    );
}
#[test]
fn quotations_share_bindings_and_recursion_gets_a_fresh_frame() {
    success(
        run(": main ( -- ) 1 let x [ 2 let x ] call x int_to_str print ;"),
        "2",
    );
    success(run(": walk ( Int -- ) let n n 0 > [ n 1 - recurse n int_to_str print ] [ ] choose call ; : main ( -- ) 3 walk ;"),"123");
    success(run(": apply ( Int [ Int -- Int ] -- Int ) call ; : main ( -- ) 7 [ ] apply int_to_str print ;"),"7");
    for source in [
        ": main ( -- ) [ x drop ] 1 let x call ;",
        ": a ( -- ) 1 let x ; : main ( -- ) x drop ;",
        ": main ( -- ) true [ 1 let x ] [ ] choose call x drop ;",
    ] {
        failure(run(source), "未定義の語または未束縛");
    }
    failure(run(": main ( -- ) 1 let x \"bad\" let x ;"), "型不一致");
    failure(
        run(": main ( -- ) true [ 1 ] [ \"s\" ] choose drop ;"),
        "型不一致",
    );
    failure(run(": main ( -- ) 1 call ;"), "引用が必要");
}
#[test]
fn imports_are_explicit_exist_and_reject_cycles_and_wrong_main() {
    let files = Files::new(&[
        (
            "main.stax",
            "use \"lib.stax\" ( value ) ; : main ( -- ) value int_to_str print ;",
        ),
        (
            "lib.stax",
            ": private ( -- Int ) 9 ; : value ( -- Int ) private ;",
        ),
    ]);
    success(files.run(&[]), "9");
    fs::write(
        files.0.join("main.stax"),
        "use \"lib.stax\" ( value ) ; : main ( -- ) private drop ;",
    )
    .unwrap();
    failure(files.run(&[]), "未定義の語");
    fs::write(files.0.join("main.stax"), "use \"lib.stax\" ( missing ) ;").unwrap();
    failure(files.run(&[]), "取込み名が実在しない");
    fs::write(files.0.join("main.stax"), "use \"absent.stax\" ( value ) ;").unwrap();
    failure(files.run(&[]), "取込みファイルが欠落");
    fs::write(files.0.join("main.stax"), "use \"lib.stax\" ( value ) ;").unwrap();
    fs::write(
        files.0.join("lib.stax"),
        "use \"main.stax\" ( value ) ; : value ( -- Int ) 1 ;",
    )
    .unwrap();
    failure(files.run(&[]), "use の循環");
    failure(run(": main ( Int -- ) drop ;"), "main の効果");
    failure(run("macro main ( -- [ -- ] ) [ ] ;"), "main の効果");
}
#[test]
fn macros_expand_before_execution_and_keep_runtime_free_bindings() {
    failure(
        run("macro constant ( -- [ -- Int ] ) 1 let n [ n ] ; : main ( -- ) constant call drop ;"),
        "捕捉値は引用",
    );
    failure(
        run("macro bind ( [ -- ] -- [ -- ] ) let q [ [ ] let q ] ; : main ( -- ) [ ] bind call ;"),
        "捕捉名",
    );
    let source="macro unless ( [ -- ] -- [ Bool -- ] ) let body [ not body [ ] choose call ] ; : main ( -- ) 8 let x false [ x int_to_str print ] unless call ;";
    success(run(source), "8");
    let files = Files::new(&[("main.stax", source)]);
    let expanded = files.run(&["--expand"]);
    assert!(expanded.status.success());
    let expanded = String::from_utf8(expanded.stdout).unwrap();
    assert!(!expanded.contains("g_name: \"unless\""));
    assert!(expanded.contains("Quotation"));
    failure(
        run("macro noisy ( [ -- ] -- [ -- ] ) \"compile\" print drop ; : main ( -- ) [ ] noisy call ;"),
        "宣言効果",
    );
    failure(run("macro noisy ( [ -- ] -- [ -- ] ) let q \"compile\" print q ; : main ( -- ) [ ] noisy call ;"),"macro 展開時の I/O");
    failure(
        run("macro identity ( [ -- ] -- [ -- ] ) ; : main ( -- ) [ ] let q q identity call ;"),
        "引用リテラル",
    );
    failure(
        run("macro identity ( [ -- ] -- [ -- ] ) ; : main ( -- ) [ ] let identity ;"),
        "macro 名",
    );
}
#[test]
fn arithmetic_strings_resource_failures_and_exit_codes() {
    success(
        run(r#": main ( -- ) -7 3 / int_to_str print " " print ;"#),
        "-2 ",
    );
    success(
        run(": main ( -- ) \"😀\\n\" str_len int_to_str print ;"),
        "2",
    );
    success(
        run(": main ( -- ) 1.5 2.5 + 4.0 = [ \"ok\" print ] [ ] choose call ;"),
        "ok",
    );
    for (source, reason) in [
        (": main ( -- ) 1 0 / drop ;", "0 除算"),
        (": main ( -- ) 1 0 mod drop ;", "0 除算"),
        (
            ": main ( -- ) 9223372036854775807 1 + drop ;",
            "整数 overflow",
        ),
        (
            ": main ( -- ) -9223372036854775808 neg drop ;",
            "整数 overflow",
        ),
        (
            ": main ( -- ) -9223372036854775808 -1 / drop ;",
            "整数 overflow",
        ),
        (
            ": main ( -- ) -9223372036854775808 -1 mod drop ;",
            "整数 overflow",
        ),
        (": main ( -- ) 9223372036854775808 drop ;", "i64 の範囲外"),
        (": main ( -- ) 1e308 1e308 * drop ;", "有限でない"),
        (": main ( -- ) 1e999 drop ;", "範囲外"),
        (": main ( -- ) \"before\" print 1 0 / drop ;", "0 除算"),
    ] {
        failure(run(source), reason);
    }
    let forever = Files::new(&[(
        "main.stax",
        ": forever ( -- ) [ recurse ] call ; : main ( -- ) forever ;",
    )]);
    failure(forever.run(&["--max-steps", "10"]), "実行ステップ上限");
    failure(forever.run(&["--max-depth", "3"]), "呼出し深さ上限");
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_stax"))
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn bounded_machine_runs_finite_loops_without_a_jvm_and_stops_infinite_recursion() {
    let files=Files::new(&[("main.stax",": times ( Int [ -- ] -- ) let body let n n 0 > [ body call n 1 - body recurse ] [ ] choose call ; : main ( -- ) 200 [ ] times \"done\" print ;")]);
    // 実行時の PATH に Java その他の実行ファイルを置かない。
    let output = Command::new(env!("CARGO_BIN_EXE_stax"))
        .env("PATH", &files.0)
        .arg(files.0.join("main.stax"))
        .output()
        .unwrap();
    success(output, "done");
    fs::write(
        files.0.join("main.stax"),
        ": forever ( -- ) recurse ; : main ( -- ) forever ;",
    )
    .unwrap();
    failure(files.run(&[]), "呼出し深さ上限");
    for args in [
        &["--max-steps", "0"][..],
        &["--max-depth", "4097"][..],
        &["--check", "--expand"][..],
    ] {
        assert_eq!(files.run(args).status.code(), Some(2));
    }
    fs::write(files.0.join("main.stax"), ": main ( -- ) [ ;").unwrap();
    failure(files.run(&[]), "構文エラー");
}

#[test]
fn quotation_type_graphs_are_bounded_and_merges_do_not_form_long_chains() {
    let mut source = String::from(": main ( -- ) [ dup ] ");
    source.push_str(&" true swap [ dup ] choose ".repeat(5000));
    source.push_str(" drop ;");
    success(run(&source), "");
    let mut source = String::from("type T0 = Int ;\n");
    for i in 1..=70 {
        source.push_str(&format!("type T{i} = [ -- T{} ] ;\n", i - 1));
    }
    source.push_str(": main ( -- ) ;");
    failure(run(&source), "引用型の深さ上限");
    let types = "Int ".repeat(1000);
    let source = format!(
        ": identity ( {types} -- {types} ) ; : main ( -- ) {} {} ;",
        "1 ".repeat(1000),
        "identity ".repeat(1000)
    );
    failure(run(&source), "型検査の操作上限");
}

#[test]
fn choose_keeps_inputs_required_by_either_branch() {
    for source in [
        ": main ( -- ) false [ ] [ 1 + ] choose call ;",
        ": main ( -- ) true [ 1 + ] [ ] choose call ;",
    ] {
        let files = Files::new(&[("main.stax", source)]);
        failure(files.run(&["--check"]), "スタック深さ");
    }
    success(
        run(r#": main ( -- ) 7 false [ ] [ 1 + ] choose call int_to_str print ;"#),
        "8",
    );
    success(
        run(r#": main ( -- ) 7 true [ ] [ 1 + ] choose call int_to_str print ;"#),
        "7",
    );
}

#[test]
fn quote_assignment_cannot_weaken_the_receivers_effect() {
    for source in [
        ": run ( [ -- ] -- ) call ; : main ( -- ) [ 1 + ] run ;",
        ": main ( -- ) [ ] let f [ 1 + ] let f f call ;",
        ": main ( -- ) [ ] let f [ f call ] let g [ 1 + ] let f g call ;",
        "struct F { q: [ -- ] } : main ( -- ) [ 1 + ] F drop ;",
        "struct F { q: [ -- ] } : main ( -- ) [ ] F [ 1 + ] !q drop ;",
    ] {
        let files = Files::new(&[("main.stax", source)]);
        failure(files.run(&["--check"]), "引用の入力効果");
    }
    success(
        run(
            r#": apply ( Int [ Int -- Int ] -- Int ) call ; : main ( -- ) 7 [ ] apply int_to_str print ;"#,
        ),
        "7",
    );
}

#[test]
fn newly_bound_quote_locals_keep_one_type_across_the_shared_word_frame() {
    let source = r#": main ( -- ) [ 1 let x [ x ] ] call let f [ "bad" let x ] call f call int_to_str print ;"#;
    let files = Files::new(&[("main.stax", source)]);
    failure(files.run(&["--check"]), "型不一致");
    success(
        run(
            r#": main ( -- ) [ 1 let x [ x ] ] call let f [ 2 let x ] call f call int_to_str print ;"#,
        ),
        "2",
    );
    failure(
        run(r#": main ( -- ) [ 1 let x ] drop "bad" let x ;"#),
        "型不一致",
    );
}
