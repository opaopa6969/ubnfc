// tree-sitter の判定と計測を 1 本にしたドライバ（issue #26）。
//
//   ts_runner check  <file>...            … 1 行 1 ファイルで TSV: path<TAB>accept|reject<TAB>理由
//   ts_runner bench  <mode> <runs> <sec> <file>...
//        mode = recognize | walk
//        recognize … ts_parser_parse_string して木を捨てる（CST は必ず作られる）
//        walk      … 上に加えて TSTreeCursor で全ノードを走査する
//        出力 CSV: impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s
//
// 受理条件: ルートにエラーが無く（ts_node_has_error）、ルートが入力全体を覆っていること。
// tree-sitter はエラー回復を持つので、parse が返ったことは受理を意味しない。
// 入力は byte 列として渡す（長さを明示するので NUL も通る）。UTF-8 として不正な
// byte を tree-sitter がどう見るかは実測して報告に書く。
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <tree_sitter/api.h>

const TSLanguage *LANGUAGE_ENTRY(void);

static char *read_file(const char *path, size_t *len) {
  FILE *f = fopen(path, "rb");
  if (!f) return NULL;
  fseek(f, 0, SEEK_END);
  long n = ftell(f);
  fseek(f, 0, SEEK_SET);
  char *buf = malloc((size_t)n + 1);
  if (!buf) { fclose(f); return NULL; }
  size_t got = fread(buf, 1, (size_t)n, f);
  fclose(f);
  buf[got] = '\0';
  *len = got;
  return buf;
}

static double now_sec(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

static size_t walk(TSNode root) {
  TSTreeCursor cursor = ts_tree_cursor_new(root);
  size_t count = 0;
  int descended = 1;
  for (;;) {
    count++;
    if (descended && ts_tree_cursor_goto_first_child(&cursor)) continue;
    if (ts_tree_cursor_goto_next_sibling(&cursor)) { descended = 1; continue; }
    for (;;) {
      if (!ts_tree_cursor_goto_parent(&cursor)) { ts_tree_cursor_delete(&cursor); return count; }
      if (ts_tree_cursor_goto_next_sibling(&cursor)) break;
    }
    descended = 1;
  }
}

static int cmp_double(const void *a, const void *b) {
  double x = *(const double *)a, y = *(const double *)b;
  return x < y ? -1 : (x > y ? 1 : 0);
}

int main(int argc, char **argv) {
  if (argc < 3) { fprintf(stderr, "usage: ts_runner check <file>... | ts_runner bench <mode> <runs> <sec> <file>...\n"); return 2; }
  TSParser *parser = ts_parser_new();
  ts_parser_set_language(parser, LANGUAGE_ENTRY());

  if (strcmp(argv[1], "check") == 0) {
    for (int i = 2; i < argc; i++) {
      size_t len = 0;
      char *text = read_file(argv[i], &len);
      if (!text) { printf("%s\treject\tread-error\n", argv[i]); continue; }
      TSTree *tree = ts_parser_parse_string(parser, NULL, text, (uint32_t)len);
      const char *verdict = "accept", *reason = "";
      if (!tree) { verdict = "reject"; reason = "parse-returned-null"; }
      else {
        TSNode root = ts_tree_root_node(tree);
        if (ts_node_has_error(root)) { verdict = "reject"; reason = "has-error"; }
        else if (ts_node_end_byte(root) != (uint32_t)len) { verdict = "reject"; reason = "unconsumed-input"; }
        else if (ts_node_is_missing(root)) { verdict = "reject"; reason = "missing-root"; }
      }
      printf("%s\t%s\t%s\n", argv[i], verdict, reason);
      if (tree) ts_tree_delete(tree);
      free(text);
      // 入力ごとに状態を持ち越さない（増分解析は使わない）。
      ts_parser_reset(parser);
    }
    ts_parser_delete(parser);
    return 0;
  }

  if (strcmp(argv[1], "bench") != 0 || argc < 6) { fprintf(stderr, "bad arguments\n"); return 2; }
  const char *mode = argv[2];
  int runs = atoi(argv[3]);
  double run_seconds = atof(argv[4]);
  int do_walk = strcmp(mode, "walk") == 0;
  printf("impl,input,bytes,mode,iterations,runs,median_ms,min_ms,max_ms,us_per_byte,mb_per_s\n");
  for (int i = 5; i < argc; i++) {
    size_t len = 0;
    char *text = read_file(argv[i], &len);
    if (!text) { fprintf(stderr, "read error: %s\n", argv[i]); return 2; }
    // 1 回の所要から反復数を決める（Rust 側 bench_util.rs と同じ作法）。
    double t0 = now_sec();
    TSTree *probe = ts_parser_parse_string(parser, NULL, text, (uint32_t)len);
    TSNode proot = ts_tree_root_node(probe);
    if (ts_node_has_error(proot)) { fprintf(stderr, "input has error: %s\n", argv[i]); return 2; }
    if (do_walk) walk(proot);
    double per = now_sec() - t0;
    ts_tree_delete(probe);
    if (per < 1e-9) per = 1e-9;
    long iterations = (long)(run_seconds / per);
    if (iterations < 3) iterations = 3;
    if (iterations > 200000) iterations = 200000;
    long warmup = iterations / 2;
    if (warmup < 3) warmup = 3;
    if (warmup > 2000) warmup = 2000;
    for (long w = 0; w < warmup; w++) {
      TSTree *t = ts_parser_parse_string(parser, NULL, text, (uint32_t)len);
      if (do_walk) walk(ts_tree_root_node(t));
      ts_tree_delete(t);
    }
    double *samples = malloc(sizeof(double) * (size_t)runs);
    for (int r = 0; r < runs; r++) {
      double start = now_sec();
      for (long it = 0; it < iterations; it++) {
        TSTree *t = ts_parser_parse_string(parser, NULL, text, (uint32_t)len);
        if (do_walk) walk(ts_tree_root_node(t));
        ts_tree_delete(t);
      }
      samples[r] = (now_sec() - start) * 1e3 / (double)iterations;
    }
    qsort(samples, (size_t)runs, sizeof(double), cmp_double);
    double median = samples[runs / 2], min = samples[0], max = samples[runs - 1];
    const char *base = strrchr(argv[i], '/');
    base = base ? base + 1 : argv[i];
    printf("tree-sitter,%s,%zu,%s,%ld,%d,%.6f,%.6f,%.6f,%.4f,%.2f\n",
           base, len, do_walk ? "CST+走査" : "CST構築", iterations, runs, median, min, max,
           median * 1e3 / (double)(len ? len : 1),
           (double)len / (median * 1e-3) / 1e6);
    free(samples);
    free(text);
  }
  ts_parser_delete(parser);
  return 0;
}
