/*
 * Acceptance + speed driver for the tree-sitter port of P4 (issue #27).
 *
 * Build (see README.md):
 *   cc -O2 -I <ts>/lib/include -I <ts>/lib/src <ts>/lib/src/lib.c \
 *      <out>/src/parser.c driver.c -o p4ts -DP4_TS_LANG=tree_sitter_tinyexpression_p4
 *
 * tree-sitter has no recognition-only mode: ts_parser_parse always materialises the
 * CST. "recognition" below is therefore parse + ts_node_has_error + free, and "tree"
 * is the same plus a full depth-first walk of every node. Both include CST
 * construction; the column headers in the report say so.
 *
 * Syntax only: P4's scope annotations have no tree-sitter counterpart.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <tree_sitter/api.h>

#ifndef P4_TS_LANG
#define P4_TS_LANG tree_sitter_tinyexpression_p4
#endif
extern const TSLanguage *P4_TS_LANG(void);

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e3 + ts.tv_nsec / 1e6;
}

static char *slurp(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) { perror(path); exit(2); }
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc((size_t)n + 1);
    if (fread(buf, 1, (size_t)n, f) != (size_t)n) { perror(path); exit(2); }
    buf[n] = 0;
    fclose(f);
    *len = (size_t)n;
    return buf;
}

/* Full DFS over the CST; returns the node count so the walk cannot be optimised away. */
static unsigned long walk(TSNode root) {
    TSTreeCursor cursor = ts_tree_cursor_new(root);
    unsigned long count = 0;
    int descend = 1;
    for (;;) {
        count++;
        if (descend && ts_tree_cursor_goto_first_child(&cursor)) { continue; }
        if (ts_tree_cursor_goto_next_sibling(&cursor)) { descend = 1; continue; }
        if (!ts_tree_cursor_goto_parent(&cursor)) { break; }
        descend = 0;
        if (ts_tree_cursor_goto_next_sibling(&cursor)) { descend = 1; continue; }
        /* keep climbing */
        for (;;) {
            if (!ts_tree_cursor_goto_parent(&cursor)) { goto done; }
            if (ts_tree_cursor_goto_next_sibling(&cursor)) { descend = 1; break; }
        }
    }
done:
    ts_tree_cursor_delete(&cursor);
    return count;
}

static int cmp_double(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    return x < y ? -1 : x > y ? 1 : 0;
}

static double load1(void) {
    FILE *f = fopen("/proc/loadavg", "r");
    double v = -1;
    if (f) { if (fscanf(f, "%lf", &v) != 1) { v = -1; } fclose(f); }
    return v;
}

/* one parse; returns 1 when accepted. Sets *nodes when walking. */
static int parse_once(TSParser *parser, const char *text, size_t len, int do_walk, unsigned long *nodes) {
    TSTree *tree = ts_parser_parse_string(parser, NULL, text, (uint32_t)len);
    TSNode root = ts_tree_root_node(tree);
    int ok = !ts_node_has_error(root) && ts_node_end_byte(root) == (uint32_t)len;
    if (do_walk) { *nodes += walk(root); }
    ts_tree_delete(tree);
    return ok;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: %s accept <dir> <name>... | bench <dir> <runs> <name>...\n", argv[0]);
        return 2;
    }
    const char *cmd = argv[1];
    const char *dir = argv[2];
    TSParser *parser = ts_parser_new();
    ts_parser_set_language(parser, P4_TS_LANG());

    if (strcmp(cmd, "accept") == 0) {
        for (int i = 3; i < argc; i++) {
            char path[4096];
            snprintf(path, sizeof path, "%s/%s.tiny", dir, argv[i]);
            size_t len; char *text = slurp(path, &len);
            unsigned long nodes = 0;
            int ok = parse_once(parser, text, len, 1, &nodes);
            printf("%s\t%zu\t%s\t%lu\n", argv[i], len, ok ? "true" : "false", nodes);
            free(text);
        }
    } else {
        int runs = atoi(argv[3]);
        if (runs < 3) { runs = 7; }
        printf("tool\tfixture\tbytes\tmode\truns\titers\tmedian_ms\tmin_ms\tload1\tnodes\n");
        for (int i = 4; i < argc; i++) {
            char path[4096];
            snprintf(path, sizeof path, "%s/%s.tiny", dir, argv[i]);
            size_t len; char *text = slurp(path, &len);
            for (int do_walk = 0; do_walk <= 1; do_walk++) {
                unsigned long nodes = 0;
                if (!parse_once(parser, text, len, do_walk, &nodes)) {
                    fprintf(stderr, "%s: rejected\n", argv[i]);
                    return 1;
                }
                /* size the run from a warm parse, not from the first one */
                for (int w = 0; w < 3; w++) { parse_once(parser, text, len, do_walk, &nodes); }
                double t0 = now_ms();
                parse_once(parser, text, len, do_walk, &nodes);
                double per = now_ms() - t0;
                if (per <= 0) { per = 1e-6; }
                int iters = (int)(500.0 / per);
                if (iters < 3) { iters = 3; }
                if (iters > 20000) { iters = 20000; }
                for (int w = 0; w < iters / 2 && w < 2000; w++) { parse_once(parser, text, len, do_walk, &nodes); }
                double *samples = malloc(sizeof(double) * (size_t)runs);
                for (int r = 0; r < runs; r++) {
                    double start = now_ms();
                    for (int k = 0; k < iters; k++) { parse_once(parser, text, len, do_walk, &nodes); }
                    samples[r] = (now_ms() - start) / iters;
                }
                qsort(samples, (size_t)runs, sizeof(double), cmp_double);
                unsigned long one = 0;
                parse_once(parser, text, len, 1, &one);
                printf("tree-sitter\t%s\t%zu\t%s\t%d\t%d\t%.5f\t%.5f\t%.2f\t%lu\n",
                       argv[i], len, do_walk ? "tree(CST+walk)" : "recognition(CST)",
                       runs, iters, samples[runs / 2], samples[0], load1(), one);
                fflush(stdout);
                free(samples);
            }
            free(text);
        }
    }
    ts_parser_delete(parser);
    return 0;
}
