import csv
import sys
from pathlib import Path

METHODS = ('MemoTable.locate', 'Session.restore', 'Session$Match.empty', 'Session.wrap', 'Session.combine')

writer = csv.writer(sys.stdout)
writer.writerow(('fixture', 'ast', 'event', 'weight', 'leaf', 'target_frames', 'memo_caller', 'first_allocation'))
for path in sorted(Path(sys.argv[1]).glob('complex*.tsv')):
    fixture, ast = path.stem.rsplit('-', 1)
    first = True
    for line in path.read_text().splitlines()[1:]:
        event, weight, leaf, *stacks = line.split('\t')
        stack = ['.'.join(frame.split('.')[-2:]) for frame in stacks[0].split(';')] if stacks else []
        frames = set(stack)
        selected = '|'.join(method for method in METHODS if method in frames)
        memo_caller = stack[1] if len(stack) > 1 and stack[0] == 'MemoTable.locate' else ''
        writer.writerow((fixture, ast, event, weight, leaf, selected, memo_caller, int(first and event == 'alloc')))
        if event == 'alloc':
            first = False
