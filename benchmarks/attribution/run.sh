#!/usr/bin/env bash
# 自分で用意した image を IMAGE で渡す。既存 container には接続しない。
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
work=${WORK:?WORK を指定する}
image=${IMAGE:?専用の Docker image を指定する}
core=${CORE:-30}
count=${COUNT:-10}
seconds=${SECONDS_PER_CASE:-15}
out=${OUT:-$work/results}
mkdir -p "$out"
# 入力・binary は WORK と同じ絶対パスで mount する。Docker 側も同じ CPU 番号を使う。
container() { docker run --rm --cpuset-cpus="$core" --user "$(id -u):$(id -g)" -v "$work:$work" -w "$work" "$image" "$@"; }
perf_container() { docker run --rm --cpuset-cpus="$core" --cap-add PERFMON --security-opt seccomp=unconfined -v "$work:$work" -w "$work" "$image" "$@"; }
case ${1:-} in
probe)
    { command -v perf || true; command -v valgrind || true; cat /proc/sys/kernel/perf_event_paranoid; } > "$out/host-tools.txt"
    container sh -c 'uname -a; valgrind --version; perf --version; cat /proc/sys/kernel/perf_event_paranoid' > "$out/docker-tools.txt" 2>&1
    # 権限や PMU 非公開も結果として残す。host の sysctl は変更しない。
    if container perf stat -e task-clock,cycles,instructions,branches,branch-misses,cache-misses -- true > "$out/perf.txt" 2>&1; then
        echo 利用可能 >> "$out/perf.txt"
    else
        echo "利用不可（exit=$?）" >> "$out/perf.txt"
    fi
    if perf_container perf stat -e task-clock,cycles,instructions,branches,branch-misses,cache-misses -- true > "$out/perf-perfmon.txt" 2>&1; then
        echo 利用可能 >> "$out/perf-perfmon.txt"
    else
        echo "利用不可（exit=$?）" >> "$out/perf-perfmon.txt"
    fi
    container valgrind --tool=cachegrind --branch-sim=yes --cachegrind-out-file="$out/cachegrind-probe.out" /bin/true > "$out/cachegrind-probe.txt" 2>&1
    ;;
rust)
    for grammar in p4 json; do
        file=$work/inputs/complex.tiny
        [[ $grammar = json ]] && file=$work/inputs/json-100k.json
        for mode in recognize ast; do
            stem=$out/rust-$grammar-$mode
            { cat /proc/loadavg; ps -eo pid,pcpu,comm --sort=-pcpu | head -20; } > "$stem.load.txt"
            taskset -c "$core" "$work/target/release/attribution" "$grammar" "$mode" "$file" 100 "$((count * 100))" > "$stem.native.csv"
            container valgrind --tool=callgrind --cache-sim=yes --branch-sim=yes \
                --instr-atstart=no \
                --callgrind-out-file="$stem.callgrind" \
                "$work/target/release/attribution" "$grammar" "$mode" "$file" 100 "$count" > "$stem.simulated.csv" 2> "$stem.log"
            container callgrind_annotate --auto=no --inclusive=no --threshold=100 "$stem.callgrind" > "$stem.annotated.txt"
            python3 "$here/summarize.py" callgrind "$stem.callgrind" "$count" > "$stem.tsv"
        done
    done
    ;;
perf)
    # 反復を増やし、起動・暖機を含むカウンタの比率を小さくする。
    for grammar in p4 json; do
        file=$work/inputs/complex.tiny
        n=${P4_PERF_COUNT:-100000}
        [[ $grammar = json ]] && { file=$work/inputs/json-100k.json; n=${JSON_PERF_COUNT:-5000}; }
        for mode in recognize ast; do
            stem=$out/perf-$grammar-$mode
            { cat /proc/loadavg; ps -eo pid,pcpu,comm --sort=-pcpu | head -20; } > "$stem.load.txt"
            perf_container perf stat -x, -o "$stem.stat.csv" -e task-clock,cycles,instructions,branches,branch-misses,cache-misses -- \
                "$work/target/release/attribution" "$grammar" "$mode" "$file" 100 "$n" > "$stem.native.csv"
            perf_container perf record -e cpu-clock:u -F 499 --call-graph dwarf,16384 -o "$stem.data" -- \
                "$work/target/release/attribution" "$grammar" "$mode" "$file" 100 "$n" > "$stem.sampled.csv" 2> "$stem.record.log"
            perf_container perf report -i "$stem.data" --stdio --no-children --percent-limit 0 --sort symbol \
                --call-graph none --fields overhead,period,symbol > "$stem.self.txt"
            python3 "$here/summarize.py" perf "$stem.self.txt" "$n" > "$stem.tsv"
        done
    done
    ;;
java)
    for grammar in p4 json; do
        file=$work/inputs/complex.tiny
        [[ $grammar = json ]] && file=$work/inputs/json-100k.json
        for mode in recognize ast; do
            stem=$out/java-$grammar-$mode
            { cat /proc/loadavg; ps -eo pid,pcpu,comm --sort=-pcpu | head -20; } > "$stem.load.txt"
            taskset -c "$core" java -Xms512m -Xmx2g -Xss16m \
                -XX:+UnlockDiagnosticVMOptions -XX:+PrintInlining \
                -cp "$work/classes" Attribution "$grammar" "$mode" "$file" 2000 "$seconds" "$stem.jfr" > "$stem.inlining.txt" 2>&1
            jfr print --json --events jdk.ExecutionSample "$stem.jfr" > "$stem.samples.json"
            python3 "$here/summarize.py" jfr "$stem.samples.json" > "$stem.tsv"
        done
    done
    ;;
*) echo 'run.sh <probe|rust|perf|java>' >&2; exit 2 ;;
esac
