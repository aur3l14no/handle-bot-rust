import json

with open('data/polyphones.json') as f:
    j = json.load(f)

with open('data/idioms.txt') as f:
    x = [l.strip() for l in f.readlines()]

with open('data/all_idioms.txt', 'w+') as f:
    f.write('\n'.join(sorted(list(set(list(j.keys()) + x)))))
